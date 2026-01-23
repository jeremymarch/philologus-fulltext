#!/bin/bash

set -ex

rm -rf ./build
rm -rf ./out

cargo build
cargo run --bin uniffi-bindgen generate --library ./target/debug/libphilologus_fulltext.dylib --language swift --out-dir ./out

# rename modulemap
mv ./out/philologus_fulltextFFI.modulemap ./out/module.modulemap

# set IPHONEOS_DEPLOYMENT_TARGET to prevent this build error: ...was built for newer 'iOS' version (26.2) than being linked (10.0)
IPHONEOS_DEPLOYMENT_TARGET=26.2 cargo build --release --target aarch64-apple-ios
IPHONEOS_DEPLOYMENT_TARGET=26.2 cargo build --release --target aarch64-apple-ios-sim
IPHONEOS_DEPLOYMENT_TARGET=26.2 cargo build --release --target aarch64-apple-darwin

rm -rf ./build
mkdir -p ./build/Headers
cp ./out/philologus_fulltextFFI.h ./build/Headers/
cp ./out/module.modulemap ./build/Headers/

# cp ./out/philologus_fulltext.swift ./Sources/philologus_fulltextFFI/

xcodebuild -create-xcframework \
-library ./target/aarch64-apple-ios/release/libphilologus_fulltext.a -headers ./build/Headers \
-library ./target/aarch64-apple-ios-sim/release/libphilologus_fulltext.a -headers ./build/Headers \
-library ./target/aarch64-apple-darwin/release/libphilologus_fulltext.a -headers ./build/Headers \
-output ./build/libphilologus_fulltext-rs.xcframework

# ditto -c -k --sequesterRsrc --keepParent ./build/libtantivy-rs.xcframework ./build/libtantivy-rs.xcframework.zip
# checksum=$(swift package compute-checksum ./build/libtantivy-rs.xcframework.zip)
# version=$(cargo metadata --format-version 1 | jq -r --arg pkg_name "tantivy-swift" '.packages[] | select(.name==$pkg_name) .version')
# sed -i "" -E "s/(let releaseTag = \")[^\"]*(\")/\1$version\2/g" ./Package.swift
# sed -i "" -E "s/(let releaseChecksum = \")[^\"]*(\")/\1$checksum\2/g" ./Package.swift
