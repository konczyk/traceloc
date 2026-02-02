use crate::core::graph::GraphBuilder;
use crate::core::ids::NodeRegistry;
use std::io::BufReader;

pub struct IngestStats {
    pub parsed: u64,
    pub skipped: u64,
}

pub fn ingest_csv<R: std::io::Read>(
    reader: R,
    builder: &mut GraphBuilder,
    node_registry: &mut NodeRegistry,
) -> anyhow::Result<IngestStats> {
    let mut csv_reader = csv::Reader::from_reader(BufReader::new(reader));
    let mut stats = IngestStats {
        parsed: 0,
        skipped: 0,
    };

    for maybe_record in csv_reader.records() {
        match maybe_record {
            Ok(record) if record.len() == 4 => {
                let src = node_registry.get_or_insert(&record[0]);
                let dst = node_registry.get_or_insert(&record[1]);
                let amount = match record[2].parse::<u64>() {
                    Ok(res) => res,
                    Err(_) => {
                        stats.skipped += 1;
                        continue;
                    }
                };
                let timestamp = match record[3].parse::<u64>() {
                    Ok(res) => res,
                    Err(_) => {
                        stats.skipped += 1;
                        continue;
                    }
                };
                builder.add_edge(src, dst, amount, timestamp);
                stats.parsed += 1;
            }
            _ => stats.skipped += 1,
        }
    }

    anyhow::Ok(stats)
}
