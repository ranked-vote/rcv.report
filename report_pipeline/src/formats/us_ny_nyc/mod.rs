mod efficient_reader;

use crate::formats::common::CandidateMap;
use crate::model::election::{Ballot, Candidate, CandidateType, Choice, Election};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::read_dir;
use std::path::Path;
use xl::{ExcelValue, Workbook, Worksheet};

/// Create a Worksheet for NYC Excel files since the xl crate's sheets() method
/// fails to parse empty elements like <sheet></sheet> (it only handles <sheet/>)
fn create_nyc_worksheet() -> Worksheet {
    Worksheet::new(
        "rId3".to_string(),
        "Sheet1".to_string(),
        1,
        "xl/worksheets/sheet1.xml".to_string(),
        1,
    )
}

struct ReaderOptions {
    office_name: String,
    jurisdiction_name: String,
    candidates_file: String,
    cvr_pattern: String,
}

impl ReaderOptions {
    pub fn from_params(params: BTreeMap<String, String>) -> ReaderOptions {
        let office_name: String = params.get("officeName").unwrap().clone();

        let jurisdiction_name: String = params.get("jurisdictionName").unwrap().clone();

        let candidates_file: String = params.get("candidatesFile").unwrap().clone();

        let cvr_pattern: String = params.get("cvrPattern").unwrap().clone();

        ReaderOptions {
            office_name,
            candidates_file,
            jurisdiction_name,
            cvr_pattern,
        }
    }
}

pub fn read_candidate_ids(workbook: &mut Workbook) -> HashMap<u32, String> {
    let mut candidates = HashMap::new();

    // Use our helper function since sheets() doesn't work with NYC files
    let worksheet = create_nyc_worksheet();
    let mut rows = worksheet.rows(workbook);
    rows.next(); // Skip header row

    for row in rows {
        let id = match &row[0].value {
            ExcelValue::Number(n) => Some(*n as u32),
            ExcelValue::String(s) => s.parse::<u32>().ok(),
            _ => None,
        };

        if let Some(id) = id {
            if let ExcelValue::String(name) = &row[1].value {
                candidates.insert(id, name.to_string());
            }
        }
    }

    candidates
}

/// Single-pass worksheet scanner that collects all ballot data for a specific race
/// Returns (eligible_precincts, ballots, candidate_ids) to avoid scanning worksheets twice
fn scan_worksheets_for_race(
    path: &Path,
    office_name: &str,
    jurisdiction_name: &str,
    cvr_pattern: &str,
    candidates: &HashMap<u32, String>,
) -> (HashSet<String>, Vec<Ballot>, CandidateMap<u32>) {
    let mut eligible_precincts: HashSet<String> = HashSet::new();
    let mut ballots: Vec<Ballot> = Vec::new();
    let mut candidate_ids: CandidateMap<u32> = CandidateMap::new();
    lazy_static! {
        static ref COLUMN_RX: Regex =
            Regex::new(r#"(.+) Choice ([1-5]) of ([1-5]) (.+) \((\d+)\)"#).unwrap();
    }

    let file_rx = Regex::new(&format!("^{}$", cvr_pattern)).unwrap();

    // Collect all matching files first
    let matching_files: Vec<_> = read_dir(path)
        .unwrap()
        .filter_map(|file| {
            let file = file.ok()?;
            let file_name = file.file_name();
            let file_name_str = file_name.to_str()?;
            if file_rx.is_match(file_name_str) {
                Some(file.path())
            } else {
                None
            }
        })
        .collect();

    // Only log if there are many files
    if matching_files.len() > 5 {
        eprintln!("Processing {} Excel files...", matching_files.len());
    }

    // Process files in parallel
    let file_results: Vec<_> = matching_files
        .par_iter()
        .map(|file_path| {
            // Only log individual file attempts if there are few files
            if matching_files.len() <= 5 {
                eprintln!("Attempting to open file: {:?}", file_path);
            }
            let mut workbook = match Workbook::open(file_path.to_str().unwrap()) {
                Ok(wb) => wb,
                Err(e) => {
                    eprintln!("Failed to open workbook: {}", e);
                    return None;
                }
            };
            // Use our helper function since sheets() doesn't work with NYC files
            let worksheet = create_nyc_worksheet();
            let mut rows = worksheet.rows(&mut workbook);
            let first_row = rows.next().unwrap();

            let mut rank_to_col: BTreeMap<u32, usize> = BTreeMap::new();
            let mut cvr_id_col: Option<usize> = None;
            let mut precinct_col: Option<usize> = None;

            // Find the precinct column, CVR ID column, and council district columns
            for (i, col) in first_row.0.iter().enumerate() {
                if let ExcelValue::String(colname) = &col.value {
                    if colname == "Cast Vote Record" || colname == "\u{feff}Cast Vote Record" {
                        cvr_id_col = Some(i);
                    } else if colname == "Precinct" {
                        precinct_col = Some(i);
                    } else if let Some(caps) = COLUMN_RX.captures(&colname) {
                        if caps.get(1).unwrap().as_str() != office_name {
                            continue;
                        }
                        if caps.get(4).unwrap().as_str() != jurisdiction_name {
                            continue;
                        }
                        let rank: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
                        assert!((1..=5).contains(&rank));
                        rank_to_col.insert(rank, i);
                    }
                }
            }

            let mut file_ballots: Vec<Ballot> = Vec::new();
            let mut file_precincts: HashSet<String> = HashSet::new();
            let mut file_candidate_ids: CandidateMap<u32> = CandidateMap::new();

            // Process all rows in a single pass
            for row in rows {
                let mut votes: Vec<Choice> = Vec::new();
                let ballot_id =
                    if let ExcelValue::String(id) = &row[cvr_id_col.unwrap() as u16].value {
                        id.to_string()
                    } else {
                        continue; // Skip if ballot ID is not a string
                    };

                // Check if this ballot is from an eligible precinct and collect votes
                let mut has_votes = false;
                if let Some(precinct_col_idx) = precinct_col {
                    if let ExcelValue::String(precinct) = &row[precinct_col_idx as u16].value {
                        // Check if this ballot has any votes for this council district
                        for col in rank_to_col.values() {
                            if let ExcelValue::String(value) = &row[*col as u16].value {
                                if value != "undervote" && value != "overvote" && !value.is_empty()
                                {
                                    file_precincts.insert(precinct.to_string());
                                    has_votes = true;
                                    break;
                                }
                            }
                        }

                        // Only process ballots from eligible precincts
                        if !has_votes {
                            continue;
                        }
                    }
                }

                // Process votes for this ballot
                for col in rank_to_col.values() {
                    let choice = match &row[*col as u16].value {
                        ExcelValue::String(value) => {
                            if value == "undervote" {
                                Choice::Undervote
                            } else if value == "overvote" {
                                Choice::Overvote
                            } else if value == "Write-in" {
                                file_candidate_ids.add_id_to_choice(
                                    0,
                                    Candidate::new("Write-in".to_string(), CandidateType::WriteIn),
                                )
                            } else {
                                match value.parse::<u32>() {
                                    Ok(ext_id) => {
                                        match candidates.get(&ext_id) {
                                            Some(candidate_name) => {
                                                file_candidate_ids.add_id_to_choice(
                                                    ext_id,
                                                    Candidate::new(candidate_name.clone(), CandidateType::Regular),
                                                )
                                            }
                                            None => {
                                                // Candidate ID not found in candidates file - skip this vote
                                                // This can happen when ballot files reference candidates from other elections
                                                eprintln!("Warning: Candidate ID {} not found in candidates file for {} - {}, skipping vote", ext_id, office_name, jurisdiction_name);
                                                Choice::Undervote
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // Failed to parse as u32 - treat as undervote
                                        eprintln!("Warning: Failed to parse candidate ID '{}', treating as undervote", value);
                                        Choice::Undervote
                                    }
                                }
                            }
                        }
                        _ => Choice::Undervote, // Default to undervote for non-string values
                    };

                    votes.push(choice);
                }

                let ballot = Ballot::new(ballot_id, votes);
                file_ballots.push(ballot);
            }

            Some((file_ballots, file_precincts, file_candidate_ids))
        })
        .collect();

    // Combine results from parallel processing
    for result in file_results {
        if let Some((file_ballots, file_precincts, file_candidate_ids)) = result {
            ballots.extend(file_ballots);
            eligible_precincts.extend(file_precincts);
            candidate_ids.merge(file_candidate_ids);
        }
    }

    (eligible_precincts, ballots, candidate_ids)
}

pub fn nyc_ballot_reader(path: &Path, params: BTreeMap<String, String>) -> Election {
    let options = ReaderOptions::from_params(params);
    let candidates_path = path.join(&options.candidates_file);

    let mut candidates_workbook = match Workbook::open(candidates_path.to_str().unwrap()) {
        Ok(workbook) => workbook,
        Err(e) => {
            eprintln!(
                "Warning: Could not open candidates file {}: {}",
                candidates_path.display(),
                e
            );
            eprintln!("Skipping this contest due to missing data file.");
            // Return empty election for missing files
            return Election::new(vec![], vec![]);
        }
    };

    let candidates = read_candidate_ids(&mut candidates_workbook);

    // Single-pass scan to get eligible precincts, ballots, and candidate IDs
    let (eligible_precincts, ballots, candidate_ids) = scan_worksheets_for_race(
        path,
        &options.office_name,
        &options.jurisdiction_name,
        &options.cvr_pattern,
        &candidates,
    );

    eprintln!(
        "Found {} eligible precincts for {} - {}",
        eligible_precincts.len(),
        options.office_name,
        options.jurisdiction_name
    );

    eprintln!(
        "Processed {} ballots for {} - {}",
        ballots.len(),
        options.office_name,
        options.jurisdiction_name
    );

    Election::new(candidate_ids.into_vec(), ballots)
}

/// Batch reader for NYC elections that parses files once and returns elections for all contests
/// Similar to nist_batch_reader, but for NYC format
pub fn nyc_batch_reader(
    path: &Path,
    contests: Vec<(String, BTreeMap<String, String>)>,
) -> HashMap<String, Election> {
    if contests.is_empty() {
        return HashMap::new();
    }

    // All contests should use the same cvrPattern and candidatesFile
    let first_params = &contests[0].1;
    let candidates_file = first_params
        .get("candidatesFile")
        .expect("us_ny_nyc elections should have candidatesFile parameter.");
    let cvr_pattern = first_params
        .get("cvrPattern")
        .expect("us_ny_nyc elections should have cvrPattern parameter.");

    // Verify all contests share the same parameters
    let same_params = contests.iter().all(|(_, params)| {
        params.get("candidatesFile") == Some(candidates_file)
            && params.get("cvrPattern") == Some(cvr_pattern)
    });

    if !same_params {
        eprintln!(
            "Warning: Not all contests share the same cvrPattern/candidatesFile, falling back to sequential processing"
        );
        return HashMap::new();
    }

    // Parse all files once using efficient_reader
    let ballot_db = efficient_reader::read_all_nyc_data(path, candidates_file, cvr_pattern);

    // Map race keys to contest office IDs
    let mut elections_by_office: HashMap<String, Election> = HashMap::new();

    for (office_id, params) in contests {
        let office_name = params
            .get("officeName")
            .expect("us_ny_nyc elections should have officeName parameter.");
        let jurisdiction_name = params
            .get("jurisdictionName")
            .expect("us_ny_nyc elections should have jurisdictionName parameter.");

        let race_key = format!("{}|{}", office_name, jurisdiction_name);

        if let Some(election) = ballot_db.to_election(&race_key) {
            elections_by_office.insert(office_id, election);
        } else {
            // Return empty election if no ballots found for this race
            elections_by_office.insert(office_id, Election::new(vec![], vec![]));
        }
    }

    elections_by_office
}
