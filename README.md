# uiefi

for some reason i decided my vehicle for learning Rust would be a UEFI application
and i wound up writing a ui framework. i'm a little surprised at this outcome but
it's been pretty fun!

this is 100% a playgrounding project and it's mostly going on github so i can seek
feedback on my rust code. once i figured out how to write to a framebuffer i basically
ignored uefi functionality. so don't take it too seriously

<img src="https://user-images.githubusercontent.com/1198161/139126945-118f1c1b-6042-495f-9f4f-9010ecd81af7.png" width="500px">

## usage

make an `Application` and call `run_loop()` on it.

`Application` takes a `Theme` to set up colors and font sizes, an initial `Widget` to
display, and the `uefi` crate's `SystemTable<Boot>` which it takes ownership of.

`MultiWidget` is a container that can own/coordinate multiple primitive `Widget`s like
`TextArea` or `Button`. They can rotate focus between `Widget`s with ^W but otherwise
have no real functionality; they need to be wrapped up in something like a `TextInput`
to hookup things like "saving" and "cancelling".

`main.rs` right now just fullscreens a `TextInput`. you can type and then ^W to switch
to the little action menu at the bottom and use arrow keys to choose "save" or "cancel".
nothing is really hooked up to handle "saved" input and "cancel" just closes it.

run `make` with all the deps installed to build and run.

## setup

`make build` runs `cargo build`
`make makefs` builds and runs `scripts/make_esp.sh` to make the EFI System Partition
`make start` builds, makes the FS, and runs `scripts/start_efi_qemu.sh`
`make` defaults to `make start`

as-is it should start `src/main.rs` but if it brings you to an EFI shell you can run
`ls fs0:\` to see what's on the mounted ESP. `fs0:\efi_hello.efi` will run that program.

i didn't really write down deps as i set it up but the list is something like
- `qemu`
- `make`
- `parted`
- `mtools` (for `mformat` and `mcopy`)
- `rustup` + rust
- the OVMF firmware for QEMU, a port of intel's tianocore UEFI firmware
  - don't remember how i got this but https://wiki.ubuntu.com/UEFI/OVMF
  - put `OVMF_CODE.fd` and `OVMF_VARS.fd` in scripts/ovmf
  - they're not that big, i just don't know if i'm allowed to redistribute them

i'm running it in the debian-based Crostini linux VM on a chromebook. if you want to try
running it on another platform, WSL might work on Windows? `parted` is linux-only so you
need a macos alternative.

## reflections on learning Rust

so far i'm kind of surprised how quickly i've gained a modest amount of comfort writing
rust. i skimmed the book sections on the borrow checker and lifetimes but mostly have
just googled stuff and played around. it's a pretty good language; it has some pointy
syntax like c++ but a lot more of python's readability.

### my gripes

i've been told the borrow checker will be familiar to people who are used to modern c++
object ownership. i see what they mean, but it's still an adjustment for me. i've also
backed out and taken different approaches when the compiler tells me to use lifetime
annotations because i'm finding them kind of inscrutable.

c++ has a "who gives a shit" attitude towards aliasing references and i've been missing
that a little bit working with Rust. in both languages i'll sometimes write code and then
do another pass for readability, and in Rust that second pass is harder if it involves
factoring out common expressions and helper methods. there were a couple places where i
just couldn't do it. it's a consequence of the borrow checker which is, like, the flagship
feature, so it's not a bad thing but it's another adjustment.

i can't really foresee how painful integrating Rust into a mature C/C++ codebase will be.
i am expecting some hard work to reconcile the c++ codebase's object lifetimes with Rust's
requirements.

this is a good and bad thing, but the compiler is very clever. i was working on code that
i thought should fail to build: https://godbolt.org/z/3zndaaEzo while an immutable borrow
of `self` was still in scope, i was able to take another mutable borrow of `self`. turns
out the compiler is clever enough to know that, because of the return immediately
following the mutable borrow, the immutable borrow can actually have its release moved up
to before the mutable borrow since we know it will never be used again. this is cool, but
like, if i have to learn a catalogue of special cases that the compiler is smart enough to
handle, that'd be a bummer. i think Rust calls this feature "non-lexical lifetimes"

### things i'm unsure of

C++ / Rust interop. i haven't looked at https://Cxx.rs yet but i suspect it can't fully
spare a team from the serious work of reconciling C++ object lifetimes with Rust
requirements. i also don't know how painful the interop surface is; can a project adopt
Rust class-by-class or subsystem-by-subsystem? can the former be a stable state or will it
evolve into the latter?

binary size and perf. i have an informed guess as to what to expect but am not doing any
benchmarking :)

### the good

the compiler diagnostics really are impressive. i've mostly encountered errors about
missing use statements and borrow checker rejections and they are not obtuse at all. i
still don't grok lifetime annotations but i haven't really tried too hard yet.

uhh the standard library generally feels pretty good? lambda syntax beats the hell out
of c++'s and writing functional-style code is pretty clean. honestly, i might already
be better with rust's standard library than i am with c++'s

the "non-lexical lifetimes" example from the "gripes" section is also a positive thing
