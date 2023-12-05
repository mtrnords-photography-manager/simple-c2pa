# Simple-C2PA Rust Library

This project builds upon the C2PA Rust library (https://github.com/contentauth/c2pa-rs) to provide an easy solution for mobile apps to add signed C2PA actions, claims, and attestations to media files. It also includes support for generating a private key and self-signed x509 certificate entirely locally on the device.

## DISCLAIMER

This is still very much "alpha" work, and is it the process of being reorganized a bit on the interface front. However, it does function as expected, and can be used on Android via Maven (https://gitlab.com/guardianproject/proofmode/simple-c2pa-android/-/packages) and iOS via Swift Package Manager (https://gitlab.com/guardianproject/proofmode/simple-c2pa-ios).

## How To Use This

See the src/lib.rs file for the current set of capabilities we have implemented.

### Build Android

You can build the native libraries yourself using the [cargo-make crate][1] with `cargo make android-build` and `cargo make apple-build`. In order to build the Android libraries, you will need Docker installed and the [latest version of cross][2]: `cargo install cross --git https://github.com/cross-rs/cross`. In order to build for iOS, you will need a Mac with Xcode installed.

## Installing the Android Library

First, add our Maven repository to your project

	allprojects {
		repositories {
	    	...
			maven {
				url = uri("https://gitlab.com/api/v4/projects/52243488/packages/maven")
			}
	       	...
	    }
	}

Then, import the simple-c2pa library, currently at version 0.0.5.

`implementation("info.guardianproject:simple-c2pa:0.0.5")`

## Installing the iOS Library

Add the SimpleC2PA Swift package in Xcode with the repository URL https://gitlab.com/guardianproject/proofmode/simple-c2pa-ios).
