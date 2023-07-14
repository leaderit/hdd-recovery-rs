/// # Disk map
use std::io;
use std::io::prelude::*;
use std::fs::File;
use log::{ info, warn, error };

/// Disk Map structure
pub struct Map {
    map_file: File,
    buf: Vec<u8>,
    pub size: usize, // Map elements
    updated: bool,
}

/// Disk Map implementation
impl Map {    
    /// Create new Disk Map
    pub fn new(file_name: &str, size: usize ) -> Map {
        let mut buf: Vec<u8> = vec![];
        let map_file = File::options().read(true).write(true).create(true).open( file_name ).unwrap();   
        buf.resize( size / 4, 0 );
        info!("Disk Map with {} bytes for {} disk blocks allocated.", buf.len(), size );
        let mut map =  Map {
            map_file,
            buf,
            size,
            updated: false
        };
        map.read();
        map
    }

    /// Read Map from file
    pub fn read( &mut self ) -> usize {

        self.map_file.seek(io::SeekFrom::Start( 0 )).unwrap();
        let len = match self.map_file.read( &mut self.buf ) {
            Ok( n ) => {
                if n > 0 {
                    info!("Disk Map with {} bytes for {} disk blocks loaded", n, self.size );
                } else {
                    warn!("New Disk Map file created.");
                }
                n
            },
            Err( e ) => {
                error!("Read Disk Map error {}", e );
                0
            }
        };
        len
    }

    /// Write Map to file
    pub fn write(&mut self ){
        self.map_file.seek(io::SeekFrom::Start( 0 )).unwrap();
        self.map_file.write(&mut self.buf).unwrap();
        self.updated = false;
    }

    /// Write Map slice to file
    pub fn write_slice(&mut self, from: usize, to: usize ){
        if from > to { return; }
        let offset = from as u64 / 4;
        let buf_from = from / 4;
        let mut buf_to = to / 4;
        if buf_to < self.buf.len() { buf_to += 1;}

        self.map_file.seek(io::SeekFrom::Start( offset )).unwrap();
        self.map_file.write(&mut self.buf[buf_from..buf_to]).unwrap();
        self.map_file.sync_all().unwrap();
        self.updated = false;
        info!("Map file updated for blocks {}-{}", from, to );
    }

    /// Get Block state
    pub fn get( &self, idx: usize ) -> u8 {

        if idx >= self.size {
            return 0xff;
        }
        let buf_idx = idx / 4;
        let bits_offs = (idx % 4) * 2;

        let b = self.buf[ buf_idx ];
        
        (b >> bits_offs) & 0x3
    }

    /// Set block state
    pub fn set( &mut self, idx: usize, value: u8 ) {
        if idx >= self.size {
            return;
        }

        let buf_idx = idx / 4;
        let bits_offs = (idx % 4) * 2;
        let b = self.buf[ buf_idx ];
        let mask = !(0x3 << bits_offs);
        let b = (b & mask) | ((value & 0x3) << bits_offs );
        self.buf[ buf_idx ] = b;
        self.updated = true;
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        if self.updated {
            self.write();
            info!("Disk Map with {} bytes writed.", self.buf.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map() {

        let test_file: &str = "test.map";

        let mut map = Map::new( test_file, 512 * 8 );

        assert_eq!( map.size, 512 * 8 );
        assert_eq!( map.buf.len(), 1024 );

        map.set(0, 3);
        map.set(3, 3);
        map.set(4, 1);
        map.set(5, 2);
        map.set(6, 1);
        map.set(7, 2);

        assert_eq!( map.buf[0], 0b11000011 );
        assert_eq!( map.buf[1], 0b10011001 );

        assert_eq!( map.get( 0 ), 3 );
        assert_eq!( map.get( 3 ), 3 );
        assert_eq!( map.get( 4 ), 1 );
        assert_eq!( map.get( 5 ), 2 );
        assert_eq!( map.get( 6 ), 1 );
        assert_eq!( map.get( 7 ), 2 );        

        map.set(28, 1);
        map.set(35, 2);
        map.write_slice( 28, 35);
        map.set(28, 2);
        map.set(35, 1);
        assert_eq!( map.get( 28 ), 2 );
        assert_eq!( map.get( 35 ), 1 );        
        map.read();
        assert_eq!( map.get( 28 ), 1 );
        assert_eq!( map.get( 35 ), 2 );
        std::fs::remove_file(test_file).unwrap();       
    }
}
