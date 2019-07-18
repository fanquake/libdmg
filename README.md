# libdmg_rust

This is a port of some of the functionality of [planetbeing/libdmg-hfsplus](https://github.com/planetbeing/libdmg-hfsplus).

The goal is to replace the usage of [`genisoimage`](https://manpages.debian.org/stretch/genisoimage/genisoimage.1.en.html) and `libdmg-hfsplus` in the Bitcoin Core [gitian build process](https://github.com/bitcoin/bitcoin/blob/master/contrib/gitian-descriptors/gitian-osx.yml#L141). 

The reason for the port is that `libdmg-hfsplus` is seemingly unmaintained and contains various bugs; some of which have been patched in a [fork](https://github.com/theuni/libdmg-hfsplus) used by Bitcoin Core. For our usage, `genisoimage` does not create deterministic files by default, so it has also been [patched](https://github.com/bitcoin/bitcoin/blob/master/depends/patches/native_cdrkit/cdrkit-deterministic.patch). It cannot compress DMGs, hence the need for `libdmg-hfsplus`.

Ideally we could have a single, well-documented tool, that can not only create and compress DMGs from scratch, but also take care of inserting `.DS_Store` related metadata (which would remove the need for [another script](https://github.com/bitcoin/bitcoin/blob/master/contrib/macdeploy/custom_dsstore.py)).

The `DMG` format is proprietary and not well documented. As a result it has been reverse engineered by multiple parties. The following resources are useful when trying to understand it:

* [ApplePartitions.ppt](http://www.cse.scu.edu/~tschwarz/COEN252_09/PPtPre/ApplePartitions.ppt)
* http://newosxbook.com/DMG.html
* [hdiutil](https://ss64.com/osx/hdiutil.html)
* [Deterministic macOS DMG Notes](https://github.com/bitcoin/bitcoin/blob/master/doc/build-osx.md#deterministic-macos-dmg-notes)
* [Secrets of the GPT](https://developer.apple.com/library/archive/technotes/tn2166/_index.html)

## Usage

```bash
mkdir my_dmg
echo "Hello World" > my_dmg/hello.txt

# create a basic dmg containing hello.txt
hdiutil create -ov -volname "Test DMG" -srcfolder my_dmg my.dmg
> created: ..../my.dmg

# open DMG and inspect contents
hdiutil attach my.dmg

# Build binary
carog build

# Inspect with libdmg_rust
cargo run inspect my.dmg
...
Inspecting: "my.dmg"

udif: KolyBlock {
    magic: 1802464377,
    version: 4,
    header_size: 512,
    flags: 1,
    running_data_fork_offset: 0,
    data_fork_offset: 0,
    data_fork_length: 8162,
    ....
```
