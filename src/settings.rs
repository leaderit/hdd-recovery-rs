use log::{ info, error };
use clap::Parser;
use serde;
use std::fs;

const SETTINGS_DEFAULT_FILE_NAME: &str = "hdd-dump.cfg";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Name of the input file
    #[arg(short, long, value_name = "INPUT")]
    pub input: Option<String>,

    /// Name of the output file
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Name of the disk map file
    #[arg(short, long, value_name = "MAP")]
    pub map: Option<String>,

    /// Update Map file every BLOCKS
    #[arg(short, long, value_name = "BLOCKS")]
    pub update: Option<usize>,
    
    /// Name of the job file
    #[arg(short, long, value_name = "JOB FILE")]
    pub job: Option<String>,

    /// Print map file file
    #[arg(short, long, value_name = "PRINT MAP")]
    pub print: bool,
}


#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct RecoveryStep {
    pub start: usize,
    pub len: usize,
    pub rev: bool
}

impl Default for RecoveryStep {
    fn default() -> RecoveryStep {
        RecoveryStep {
            start: 0,
            len: 0,
            rev: false
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct Settings {
    pub input: String,
    pub output: String,
    pub map: String,
    pub size: usize,
    pub block_size: usize,
    pub blocks: usize,
    pub program: Vec<RecoveryStep>
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            input: "".into(),
            output: "".into(),
            map: "".into(),
            size: 0,
            block_size: 0,
            blocks: 0,
            program: vec![]            
        }
    }
}

impl Settings {
    pub fn args() -> Args {
        Args::parse()
    }

    pub fn init( file_name: Option<&str> ) -> Settings {
        let data = fs::read_to_string(file_name.unwrap_or( SETTINGS_DEFAULT_FILE_NAME )).expect("Couldn't find or load config file.");
        match serde_json::from_str(&data) {
            Ok( settings ) => {
                info!("Configuration file loaded.");
                settings
            },
            Err( e ) => {
                error!("{:?}", e);
                Settings::default()
            }            
        }
    }    
}