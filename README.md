# hpbdl
Extract HP printer ".bdl" firmware update files

This roughly interprets the necessary fields of a BDL file to split it into
individual partition "ipkg" files, and extracts the files in each of those to a
directory with the partition name.

"ipkg" does not refer to the Linux packaging format.

Some hard-coded offsets are used that possibly should be determined by many of
the unused binary fields.

Works for at least HP M604 and M607 update files. They are distributed as a
.zip containing a .bdl.
