use std::fs;
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.data_file)?;
    println!("file contents: {}", contents);

    // TODO: 
    // - list file names in config.dir.
    // - configure template/rule to determine new name of file.
    // - write function that takes a csv line and returns the data needed for 
    // the template.
    // - write function that returns the new name of the file.
    // - test all the above.
    // - write function that moves a file.
    // - integration test the actual moving according to some template config.

    Ok(())
}

pub struct Config{
    pub data_file: String,
    pub dir: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 3 {
            return Err("received incorrect number of arguments: need 2");
        }

        let data_file = args[1].clone();
        let dir = args[2].clone();

        Ok(Config{ data_file, dir })
    }
}
