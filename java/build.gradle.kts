plugins {
    application
}

group = "bot"
version = "0.1.0"

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    implementation("io.github.ccxt:ccxt:4.5.84")
}

application {
    mainClass.set("bot.Main")
}

tasks.test {
    enabled = false
}
