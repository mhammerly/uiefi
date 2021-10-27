#!/bin/bash

# https://unix.stackexchange.com/questions/281589/how-to-run-mkfs-on-file-image-partitions-without-mounting
# i think this turned to be unnecessary. qemu can just mount a dir directly lol
# but i only saw that after i got it set up this way so

script_dir="$(dirname $0)"
diskimg="$script_dir/fat32.fs"
size=$((100*(1<<20))) # desired size in bytes (100mb here)
# align to next MB (https://www.thomas-krenn.com/en/wiki/Partition_Alignment)
alignment=1048576

size=$(( (size + alignment - 1)/alignment * alignment ))
echo $size
echo $alignment

# image size is gpt + filesystem size + gpt backup
truncate -s $((size + 2*alignment)) "${diskimg}"

sudo parted --machine --script --align optimal "${diskimg}" mklabel gpt mkpart ESP "${alignment}B" '100%' set 1 boot on

mformat -i "${diskimg}"@@"${alignment}" -t $((size>>20)) -h 64 -s 32 -F -v "volname"

mcopy -i "${diskimg}"@@"${alignment}" -s $script_dir/put_on_esp/* ::
