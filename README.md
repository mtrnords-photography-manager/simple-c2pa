# Simple-C2PA Rust Library

This project builds upon the C2PA Rust library (https://github.com/contentauth/c2pa-rs) to provide an easy solution for mobile apps to add signed C2PA actions, claims, and attestations to media files. It also includes support for generating a private key and self-signed x509 certificate entirely locally on the device.

## DISCLAIMER

This is still very much "alpha" work, and is it the process of being reorganized a bit on the interface front. However, it does function as expected, and can be used on Android via Maven (https://gitlab.com/guardianproject/proofmode/simple-c2pa/-/packages) and iOS via Swift Package Manager.

## How To Use This

See the tests/examples.rs file for the current set of capabilities we have implemented.

## Installing the Android Library

First, add our Maven repository to your project

```
    allprojects {
    	repositories {
        	...
    		maven {
    			url = uri("https://gitlab.com/api/v4/projects/51891540/packages/maven")
    		}
           	...
        }
    }
```

Then, import the simple-c2pa library, currently at version 0.0.13.

`implementation("info.guardianproject:simple-c2pa:0.0.13")`

## Sample Kotlin code

```
val rootCert = createRootCertificate(null, null)
val contentCert = createContentCredentialsCertificate(rootCert, null, null)
val fileData = FileData(imagePath, null, fileName)
val cc = ContentCredentials(contentCert, fileData, null)
cc.addCreatedAssertion()
cc.embedManifest(outputPath)
```

## Installing the iOS Library

Add the SimpleC2PA Swift package in Xcode with the repository URL https://gitlab.com/guardianproject/proofmode/simple-c2pa. The current version is 0.0.13.

## Sample Swift code

```
let rootCert = try! createRootCertificate(organization: nil, validityDays: nil);
let contentCert = try! createContentCredentialsCertificate(rootCertificate: rootCert, organization: nil, validityDays: nil)
let fileData = FileData(path: imagePath, bytes: nil, fileName: filename)
let cc = ContentCredentials(certificate: contentCert, file: fileData, applicationInfo: nil)
try! cc.addCreatedAssertion()
try! cc.embedManifest(outputPath: outputPath)
```

## Build Android library

Build the native Android library yourself using [cargo-make](https://github.com/sagiegurari/cargo-make) with the command `cargo make android-build`. You will also need Docker installed and the latest version of [cross](https://github.com/cross-rs/cross).

## Build Apple library

Build the native Apple library yourself using [cargo-make](https://github.com/sagiegurari/cargo-make) with the command `cargo make apple-build`. You will need to run the command on a Mac with Xcode installed.
