import org.gradle.api.publish.maven.MavenPublication

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    `maven-publish`
}

android {
    namespace = "info.guardianproject.simple_c2pa"
    compileSdk = 33

    defaultConfig {
        minSdk = 21

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.9.0")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.8.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
    implementation("net.java.dev.jna:jna:5.13.0@aar")
}

publishing {
  publications {
    create<MavenPublication>("release") {
      artifact("$buildDir/outputs/aar/${project.name}-release.aar")
      groupId = "info.guardianproject"
      artifactId = "simple_c2pa"
      version = System.getenv("CI_COMMIT_TAG") as String? 
    }
  }
  repositories {
    maven {
      url = uri("https://gitlab.com/api/v4/projects/51891540/packages/maven")
      credentials {
        this.username =  System.getenv("MAVEN_USERNAME") as String?
        this.password =  System.getenv("MAVEN_PASSWORD") as String?
      }
    }
  }
}

