use crate::commands::report::rebuild_index;
use crate::formats::us_ny_nyc::efficient_reader::read_all_nyc_data;
use crate::model::election::{ElectionInfo, ElectionPreprocessed};
use crate::normalizers::normalize_election;
use crate::read_metadata::read_meta;
use crate::report::generate_report;
use crate::util::write_serialized;
use std::fs::create_dir_all;
use std::path::Path;

/// Efficiently process all NYC races in one pass and generate final reports
/// This skips the normalized.json.gz step entirely and generates reports directly from memory
pub fn preprocess_nyc(metadata_dir: &str, raw_data_dir: &str, report_dir: &str) {
    // Read metadata from all files in the directory
    let metadata_files: Vec<_> = read_meta(Path::new(metadata_dir)).collect();

    eprintln!("🏙️  Starting efficient NYC processing...");

    // Find NYC jurisdiction file
    let (_, nyc_jurisdiction) = metadata_files
        .iter()
        .find(|(path, _)| path.to_string_lossy().contains("nyc.json"))
        .expect("NYC jurisdiction metadata not found");

    // Process only the 2025 election
    for (election_path, election_metadata) in &nyc_jurisdiction.elections {
        if election_metadata.data_format != "us_ny_nyc" {
            continue;
        }
        
        // Skip 2021 election, only process 2025
        if election_path != "2025/07" {
            eprintln!("⏭️  Skipping election: {} (only processing 2025/07)", election_path);
            continue;
        }

        eprintln!("📅 Processing election: {}", election_path);

        // Get the raw data path for this election
        let raw_election_path = Path::new(raw_data_dir)
            .join(&nyc_jurisdiction.path)
            .join(election_path);

        if !raw_election_path.exists() {
            eprintln!(
                "⚠️  Skipping {}: raw data directory not found",
                election_path
            );
            continue;
        }

        // Extract common parameters from first contest (they should all be the same for NYC)
        let first_contest = election_metadata
            .contests
            .first()
            .expect("No contests found in election");
        let loader_params = first_contest
            .loader_params
            .as_ref()
            .expect("loader_params not found in contest");
        let candidates_file = loader_params
            .get("candidatesFile")
            .expect("candidatesFile not found in loader params");
        let cvr_pattern = loader_params
            .get("cvrPattern")
            .expect("cvrPattern not found in loader params");

        // Process all races in one efficient pass
        eprintln!("🚀 Reading all NYC data efficiently...");
        let ballot_db = read_all_nyc_data(&raw_election_path, candidates_file, cvr_pattern);

        // Generate reports directly for ALL races at once
        eprintln!("📊 Generating reports for all {} races...", ballot_db.races.len());
        for (race_key, _race_metadata) in &ballot_db.races {
            // Find the corresponding contest in metadata
            let contest = election_metadata.contests.iter()
                .find(|c| {
                    if let Some(params) = &c.loader_params {
                        let contest_race_key = format!(
                            "{}|{}",
                            params.get("officeName").map_or("", |v| v),
                            params.get("jurisdictionName").map_or("", |v| v)
                        );
                        contest_race_key == *race_key
                    } else {
                        false
                    }
                });

            if let Some(contest) = contest {
                let office = nyc_jurisdiction
                    .offices
                    .get(&contest.office)
                    .expect("Office not found in jurisdiction");

                if let Some(election) = ballot_db.to_election(race_key) {
                    eprintln!("  📊 {} -> {} ballots", office.name, election.ballots.len());

                    // Normalize the election
                    let normalized = normalize_election(
                        &election_metadata.normalization,
                        election,
                    );

                    // Create ElectionPreprocessed for report generation
                    let election_info = ElectionInfo {
                        name: office.name.clone(),
                        date: election_metadata.date.clone(),
                        data_format: election_metadata.data_format.clone(),
                        tabulation_options: election_metadata.tabulation_options.clone().unwrap_or_default(),
                        jurisdiction_path: nyc_jurisdiction.path.clone(),
                        election_path: election_path.clone(),
                        office: contest.office.clone(),
                        office_name: office.name.clone(),
                        jurisdiction_name: nyc_jurisdiction.name.clone(),
                        election_name: election_metadata.name.clone(),
                        loader_params: contest.loader_params.clone(),
                        website: None,
                    };

                    let election_preprocessed = ElectionPreprocessed {
                        info: election_info,
                        ballots: normalized,
                    };

                    // Generate the final report
                    if let Some(report) = generate_report(&election_preprocessed) {
                        // Write report to final directory
                        let report_path = Path::new(report_dir)
                            .join(&nyc_jurisdiction.path)
                            .join(election_path)
                            .join(&contest.office)
                            .join("report.json");

                        // Create directory if it doesn't exist
                        if let Some(parent) = report_path.parent() {
                            create_dir_all(parent).unwrap();
                        }

                        write_serialized(&report_path, &report);
                        eprintln!("    ✅ Generated report: {}", report_path.display());
                    } else {
                        eprintln!("    ⚠️  Could not generate report for: {}", race_key);
                    }
                } else {
                    eprintln!("  ⚠️  No ballots found for race: {}", race_key);
                }
            } else {
                eprintln!("  ⚠️  No metadata found for race: {}", race_key);
            }
        }
    }

    // Rebuild the index.json to include all generated reports
    rebuild_index(Path::new(report_dir));

    eprintln!("✅ NYC processing complete!");
}
