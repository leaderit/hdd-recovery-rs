# hdd-recovery-rs utility

Recovery disk block by block by reading every block from input, writing one into output and mark block state in map file. States are unread, recovered and error.

Recovery going under manual recovery program where you may 
select groups of blocks, lenght ones and direction.

With this options you may read only good parts and skip bad 
areas on the disk.

As result you'll get full copy of your disk as binary image.

## Usage

    hdd-recovery-rs v0.1.0
    Usage: hdd-recovery-rs [OPTIONS]

    Options:
    -c, --config <FILE>    Name of the config file
    -i, --input <INPUT>    Name of the input file
    -o, --output <OUTPUT>  Name of the output file
    -m, --map <MAP>        Name of the disk map file
    -u, --update <BLOCKS>  Update Map file every BLOCKS
    -j, --job <JOB FILE>   Name of the job file
    -p, --print            Print map file file
    -h, --help             Print help
    -V, --version          Print version

## Configuration file

Configuration file is json file with next structure:

    {
        "input":"/dev/disk4s2",
        "output":"wm.img",
        "map": "wm.map",
        "size": 524288,
        "block_size": 512,
        "blocks": 1024,
        "program":[
            { "start": 512, "len": 512, "rev": false },
            { "start": 256, "len": 256, "rev": false },
            { "start": 0, "len": 256, "rev": true }
        ]
    }

where
- input is input drive for recovery
- output is outpt file with recovered date
- map is a recovery map file with information about unreaded, recovered and error blocks
- size is a size of the disk in blocks
- block_size is a disk block size
- blocks is a total amount of blocks in the disk. It must be equal with size / block_size
- program is a recovery program in recovery job file format

## Recovery job file

Recovery job file is json file with next structure:

    [
        { "start": 0, "len": 256, "rev": true }
    ]

Recovery job load with --job or -j parameter

## Additional information

For MacOS you may use command

    sudo diskutil list physical

for get list of physical hard drivers. You must unmount the drive wich you'd like to recover. 
For MacOs you may use the command, set disk name as you choice 

    sudo diskutil unmountDisk force /dev/disk4
