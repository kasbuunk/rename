use csv;
use std::collections::HashMap;
use std::fs;

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let file_names = list_files(&config.dir);

    let csv_rows = read_csv(&config.data_file)?;

    let renamings = determine_renamings(csv_rows, file_names);

    let result = rename_all_files(&config.dir, renamings);

    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}

fn read_csv(file_name: &String) -> Result<Vec<csv::StringRecord>, Box<dyn std::error::Error>> {
    let mut rows: Vec<csv::StringRecord> = vec![];

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(file_name)?;
    for result in reader.records() {
        let record = result?;
        rows.push(record);
    }

    Ok(rows)
}

fn determine_renamings(
    csv_rows: Vec<csv::StringRecord>,
    files: Vec<String>,
) -> HashMap<String, String> {
    let mut renamings: HashMap<String, String> = HashMap::new();

    for row in csv_rows {
        let lot_number = row.get(0).expect("Malformed csv row: 0th value not found.");
        let inventory_number = row.get(8).expect("Malformed csv row: 8th value not found.");

        let object_files = filter_object_files(files.clone(), inventory_number.to_string());
        for object_file in object_files {
            let suffix = extract_file_suffix(&object_file);
            let new_name = compose_new_name(lot_number, suffix);
            renamings.insert(object_file, new_name);
        }
    }

    renamings
}

fn compose_new_name(lot_number: &str, suffix: &str) -> String {
    format!("{}_{}.jpg", lot_number, suffix)
}

// extract_file_suffix gets the number between the two periods.
fn extract_file_suffix(file_name: &str) -> &str {
    let name_parts_between_periods = file_name.split(".").collect::<Vec<&str>>();
    name_parts_between_periods[1]
}

fn rename_all_files(dir: &str, renamings: HashMap<String, String>) -> std::io::Result<()> {
    for (old_name, new_name) in &renamings {
        println!("renaming {} to {}", old_name, new_name);
        let directory = std::path::Path::new(dir);
        let old_path = directory.join(old_name);
        let new_path = directory.join(new_name);

        fs::rename(old_path, new_path)?;
    }

    Ok(())
}

pub struct Config {
    pub data_file: String,
    pub dir: String,
}

impl Config {
    pub fn new(data_file: String, dir: String) -> Config {
        Config { data_file, dir }
    }

    pub fn from_args(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 3 {
            return Err("received incorrect number of arguments: need 2");
        }

        let data_file = args[1].clone();
        let dir = args[2].clone();

        if !validate_dir(&dir) {
            return Err("given directory path is not a directory");
        }

        Ok(Config::new(data_file, dir))
    }
}

fn validate_dir(file: &str) -> bool {
    let result = fs::metadata(file);
    match result {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

fn list_files(dir: &str) -> Vec<String> {
    let mut files: Vec<String> = vec![];

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                files.push(entry.file_name().to_str().unwrap().to_owned());
            }
        }
    }

    files
}

// filter_object_files finds files prefixed with this inventory number.
fn filter_object_files(files: Vec<String>, object_id: String) -> Vec<String> {
    files
        .into_iter()
        .filter(|element| element.starts_with(&object_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end() {
        // The pattern that new files must match.
        let expression = regex::Regex::new(r"[1-9][0-9]*_[1-9][1]?.jpg").unwrap();

        let images_dir = std::path::Path::new("tests/files/");
        let test_dir = std::path::Path::new("tests/tmp/");
        let data_file = "tests/data.csv";

        let config = Config::new(
            String::from(data_file),
            String::from(test_dir.to_str().unwrap()),
        );

        // Copy tests directory to tmp.
        let _ = fs::create_dir(test_dir);

        let file_names = list_files(images_dir.to_str().unwrap());
        for file in file_names.clone() {
            let old_path = images_dir.join(&file);
            let new_path = test_dir.join(&file);

            fs::copy(old_path, new_path).unwrap();

            // Assert all old file names don't match the pattern to which the files
            // ought to be renamed.
            assert!(!expression.is_match(&file));
        }

        run(config).expect("Running failed");

        let new_file_names = list_files(test_dir.to_str().unwrap());
        assert_eq!(file_names.len(), new_file_names.len());

        // Assert all moved files have a name that matches the pattern.
        for file in new_file_names {
            assert!(expression.is_match(&file));
        }

        fs::remove_dir_all("tests/tmp").expect("Could not delete tests directory.");
    }

    #[test]
    fn determine_file_names() {
        let data = String::from("1		\"Henricus Johannes (Harrie) Kuyten, Utrecht 1883-1952 Schoorl ...\"	\"Henricus Johannes (Harrie) Kuyten, Utrecht 1883-1952 Schoorl, Beach view with various people, oil on canvas, 43 x 36 cm.\"	EUR	4000	6000	3000	00243878										\n2		\"Henricus Johannes (Harrie) Kuyten, Utrecht 1883-1952 Schoorl ...\"	\"Henricus Johannes (Harrie) Kuyten, Utrecht 1883-1952 Schoorl, Beach view, pastel drawing, dated 1951, 31,5 x 23 cm\"	EUR	500	700	380	00243880										\n3		\"Very large antique blue/white Chinese porcelain lidded vase ...\"	\"Very large antique blue/white Chinese porcelain lidded vase with decoration of floral motifs, Qing Dynasty, approx. h.59 cm.\"	EUR	2000	3000	1500	00243344										");

        let mut rows: Vec<csv::StringRecord> = vec![];

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_reader(data.as_bytes());
        for result in rdr.records() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            let record = result.unwrap();
            rows.push(record);
        }

        let file_names = vec![
            "00243878.1.jpg".to_string(),
            "00243878.2.jpg".to_string(),
            "00243878.3.jpg".to_string(),
            "00243880.1.jpg".to_string(),
            "00243880.2.jpg".to_string(),
            "00243344.1.jpg".to_string(),
            "00243344.2.jpg".to_string(),
            "00243344.3.jpg".to_string(),
        ];

        let renamings = determine_renamings(rows, file_names);

        let expected_renamings: HashMap<String, String> = HashMap::from([
            ("00243878.1.jpg".to_string(), "1_1.jpg".to_string()),
            ("00243878.2.jpg".to_string(), "1_2.jpg".to_string()),
            ("00243878.3.jpg".to_string(), "1_3.jpg".to_string()),
            ("00243880.1.jpg".to_string(), "2_1.jpg".to_string()),
            ("00243880.2.jpg".to_string(), "2_2.jpg".to_string()),
            ("00243344.1.jpg".to_string(), "3_1.jpg".to_string()),
            ("00243344.2.jpg".to_string(), "3_2.jpg".to_string()),
            ("00243344.3.jpg".to_string(), "3_3.jpg".to_string()),
        ]);

        assert_eq!(expected_renamings, renamings)
    }

    #[test]
    fn filter_object_files_test() {
        let files = list_files("tests/files");
        let object_id = String::from("00243344");

        //TODO: refactor vector to a collection that is unordered (hash map?).
        assert_eq!(
            vec![
                "00243344.6.jpg",
                "00243344.7.jpg",
                "00243344.5.jpg",
                "00243344.4.jpg",
                "00243344.1.jpg",
                "00243344.3.jpg",
                "00243344.2.jpg",
            ],
            filter_object_files(files, object_id)
        );
    }

    #[test]
    fn read_dir_contents() {
        let dir = "tests/files";

        assert_eq!(
            vec![
                "00243880.6.jpg",
                "00243880.4.jpg",
                "00243880.5.jpg",
                "00243880.1.jpg",
                "00243880.2.jpg",
                "00243880.3.jpg",
                "00243344.6.jpg",
                "00243344.7.jpg",
                "00243878.1.jpg",
                "00243344.5.jpg",
                "00243878.3.jpg",
                "00243878.2.jpg",
                "00243344.4.jpg",
                "00243878.6.jpg",
                "00243878.7.jpg",
                "00243344.1.jpg",
                "00243878.5.jpg",
                "00243344.3.jpg",
                "00243344.2.jpg",
                "00243878.4.jpg",
            ],
            list_files(dir),
        );
    }

    #[test]
    fn dir_exists() {
        let dir_existing = "src";
        let dir_not_existing = "my_random_nonexistent_dirname";

        assert!(validate_dir(dir_existing));
        assert!(!validate_dir(dir_not_existing));
    }
}
