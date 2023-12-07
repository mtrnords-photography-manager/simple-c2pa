// swift-tools-version:5.5
import PackageDescription

let package = Package(
    name: "SimpleC2PA",
    platforms: [
        .iOS(.v15),
    ],
    products: [
        .library(
            name: "SimpleC2PA",
            targets: ["SimpleC2PA"]
        ),
    ],
    dependencies: [],
    targets: [
        .binaryTarget(
            name: "SimpleC2PAFramework",
             url: "https://gitlab.com/api/v4/projects/51891540/packages/generic/simple_c2pa/0.0.12/SimpleC2PA.xcframework.zip",
             checksum: "29d01a14cc76ddf53a1517194ad00f588b964aad78c3836054b81938a38637ee"
        ),
        .target(
            name: "SimpleC2PA",
             path: "apple/src",
            dependencies: [ "SimpleC2PAFramework" ]             
        ),
    ]
)
