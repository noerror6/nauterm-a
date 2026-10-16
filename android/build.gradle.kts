allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

// The jni Flutter plugin requests Flutter's default NDK 28. Use the installed
// NDK 27 for all plugin projects as well as the app.
subprojects {
    afterEvaluate {
        if (name == "jni") {
            extensions.findByType(com.android.build.gradle.LibraryExtension::class.java)
                ?.ndkVersion = "27.2.12479018"
        }
    }
}

val newBuildDir = rootProject.layout.buildDirectory.dir("../../build").get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    layout.buildDirectory.value(newBuildDir.dir(project.name))
    evaluationDependsOn(":app")
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
