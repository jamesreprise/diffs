# Diffs
## Multi-threaded directory diff'er
Diffs uses rayon and SeaHash to get the difference between two directories by 'flattening' all subdirectories and comparing all the files and their hashes.

Syntax: `./diffs <old directory> <new directory>`