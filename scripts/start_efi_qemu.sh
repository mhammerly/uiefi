#!/bin/bash

script_dir="$(dirname $0)"

# need to use the expanded drive/device pair for rootfs in order to specify
# boot order in a way UEFI/OVMF respects.
qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly,file=$script_dir/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=$script_dir/ovmf/OVMF_VARS.fd \
    -drive file=$script_dir/fat32.fs,index=0,media=disk,if=none,id=rootfs \
    -device ide-hd,drive=rootfs,bootindex=1
