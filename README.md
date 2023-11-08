# Simple-C2PA Rust Library

This project builds upon the C2PA Rust library (https://github.com/contentauth/c2pa-rs) to provide an easy solution for mobile apps to add signed C2PA actions, claims, and attestations to media files. It also includes support for generating a private key and self-signed x509 certificate entirely locally on the device.

## DISCLAIMER

This is still very much "alpha" work, and is it the process of being reorganized a bit on the interface front. However, it does function as expected, and can be easily used on Android through published Maven dependencies.

## How To Use This

See the src/lib.rs file for the current set of capabilities we have implemented. You can also see the src/android.rs for the JNI interface that can be called from Java/Kotlin on Android, and the src/apple.rs for iOS devices.

You can build the native libraries yourself using the Makefile and 'make android' or 'make ios' commands.

## Sample Code for Android

There is also a sample project for Android, with the C2paSampleActivity.kt sample activity. Below is an example set of calls:

	var certPath = File(filesDir,"cr.cert")
        var certKey = File(filesDir, "cr.key")

        var imagePath = File(getExternalFilesDir(null), "test.jpg")
        var identity = "joe@https://instagram.com/joeschmo"; //string needs name@uri/identity format
        var fingerprint = "12345678"
        var outputPath = File(getExternalFilesDir(null), "test-c2pa" + Date().time + ".jpg")

        if (!certPath.exists() || !certKey.exists())
            C2paJNI.generateCredentials(certPath.absolutePath, certKey.absolutePath, fingerprint)

        var isDirectCapture = false;
        var allowMachineLearning = true;

        C2paJNI.addAssert(certPath.absolutePath, certKey.absolutePath, imagePath.absolutePath, identity, fingerprint, isDirectCapture, allowMachineLearning, outputPath.absolutePath)


## Android Library

First, add our GPMaven repository to your project

	allprojects {
    		repositories {
		...
        	maven { url "https://raw.githubusercontent.com/guardianproject/gpmaven/master" }
       		...
    		}
	}

Then, import the proofmode-c2pa library, currently at version 0.3

	implementation("org.proofmode:proofmode-c2pa:0.3")

## iOS Library

Information on how to build and use the library for iOS will be here shortly.
