mod efficient_reader;

use crate::model::election::Election;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

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
        crate::log_warn!(
            "Not all contests share the same cvrPattern/candidatesFile, falling back to sequential processing"
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
