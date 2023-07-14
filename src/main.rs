mod settings;
mod map;

use crate::settings::*;
use crate::map::*;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use log::{info, warn, error};
use std::fs;

/// Print Disk map
fn print_map( map: &Map) -> u8 {
    let columns = 64;           // display columns
    let rows = 64;              // display rows
    let div = map.size / ( columns * rows ) + 1;

    let mut col = 0;
    let mut sub = 0;
    let mut row = 0;
    let mut c = ' ';

    for idx in 0..map.size {
        if col == 0 {
            print!("{:12?}  ", idx);
        }
        let b = map.get( idx );
        if sub == 0 {c = '.'};
        if sub < div {
            if b == 1 && c == '.' {c='+'};
            if b == 2 { c='E'}
            sub +=1;
        }
        if sub == div {
            print!("{} ", c);
            col +=1;
            sub = 0;
            if col % 4 == 0 { print!(" "); }

        }
        if col >= columns {
            println!("");
            col = 0;
            row +=1;
        }
    }
    row
}

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> io::Result<()> {

    println!("{} v{}", APP_NAME, APP_VERSION);
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    let args = Settings::args();
    let mut cfg = Settings::init( args.config.as_deref() );

    if cfg.size / cfg.block_size != cfg.blocks {
        error!("{} disc blocks with {} block size don't correlate with {} disk size.", 
        cfg.blocks, cfg.block_size, cfg.size);
        panic!("Disk configuration error.");
    }

    let program: Vec<RecoveryStep> = match args.job {
        Some( file_name ) => {
            let data = fs::read_to_string(file_name).expect("Couldn't find or load job program file.");
            let program = match serde_json::from_str(&data) {
                Ok( prog ) => {
                    info!("Job program file loaded.");
                    prog
                },
                Err( e ) => {
                    error!("{:?}", e);
                    panic!("Job program format error");
                }            
            };
            program
        },
        None => cfg.program
    };

    cfg.input = args.input.clone().unwrap_or( cfg.input );
    cfg.output = args.output.clone().unwrap_or( cfg.output );
    cfg.map = args.map.clone().unwrap_or( cfg.map );
    let args_update = args.update.unwrap_or(0);

    let mut buffer: Vec<u8> = vec![0; cfg.block_size];
    let mut map = Map::new( &cfg.map, cfg.blocks );

    if args.print {
        print_map( &map );
        return Ok(());
    }

    let mut reader = match File::open( &cfg.input ) {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the Input file {}: {:?}", &cfg.input, error),
    };
    let mut writer = match File::options().write(true).create(true).open( &cfg.output ){
        Ok(file) => file,
        Err(error) => panic!("Problem opening the Output file {}: {:?}", &cfg.output, error),
    };

    for task in program {
        info!("{:?}", task);
        let offs = (task.start * cfg.block_size) as u64;
        let len = reader.seek(io::SeekFrom::Start( offs ))?;
        if len != offs { error!("seek error task={} seek={}", offs, len);}
        let len = writer.seek(io::SeekFrom::Start( offs ))?;
        if len != offs { error!("seek error task={} seek={}", offs, len);}

        let ( start, end ) = if task.rev {
            // read blocks in reverse order
            ( task.len - 1, 0 )
        } else {
            // read blocks in normal order
            ( 0, task.len - 1 )
        };

        let mut block_number = start;
        let mut do_job = false;
        let mut update_count = 0;
        loop {
            let block_idx = task.start + block_number;
            if map.get( block_idx ) != 1 {
                do_job = true;
                // seek back on the block every time when reverce order
                if task.rev {
                    let offs = (block_idx * cfg.block_size) as u64;
                    let len = reader.seek(io::SeekFrom::Start( offs ))?;
                    if len != offs { error!("seek error task={} seek={}", offs, len);}
                    let len = writer.seek(io::SeekFrom::Start( offs ))?;
                    if len != offs { error!("seek error task={} seek={}", offs, len);}
                }

                match reader.read(&mut buffer) {
                    Ok( n ) => {
                        if n == 512 {
                            writer.write(&buffer)?;
                            map.set( block_idx, 1);            
                        } else {
                            error!("Error block {}", block_idx);
                            map.set( block_idx, 2);            
                        }
                    },
                    Err( _e ) => {
                        error!("Error read block {}", block_idx);
                        map.set( block_idx, 2);            
                    }
                }
                print!("Block readed={}\r", block_idx );
                // if block_number != end { print!("\r") };
                // Update Map File every args.update blocks
                if args_update > 0 {
                    update_count += 1;
                    if update_count >= args_update {
                        io::stdout().flush()?;
                        let ( from, to ) = if task.rev { 
                            ( block_idx, block_idx + args_update ) 
                        } else {
                            ( block_idx - args_update, block_idx )
                        };
                        writer.sync_all()?;
                        map.write_slice( from, to );
                        update_count = 0;    
                    }
                }
            }
            if block_number == end { break; } 
            else {
                if task.rev { block_number -=1; } else { block_number+=1; }
            }
            io::stdout().flush()?;
            // delay for debug purpose
            // std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if do_job { 
            info!("Blocks readed/writed: {}", task.len );
            if args_update > 0 {
                io::stdout().flush()?;
                let ( from, to ) = if task.rev { 
                    ( task.start, task.start + update_count - 1 ) 
                } else {
                    ( task.start + task.len - update_count, task.start + task.len - 1 )
                };
                writer.sync_all()?;
                map.write_slice( from, to );
            }
        } else {
            warn!("Nothing to do.");
        }
    }
    Ok(())    
}
