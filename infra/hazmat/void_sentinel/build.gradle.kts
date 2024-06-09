tasks.register<Exec>("hashPass") {
    doLast {
        kotlin.run {
            "./hash_pass.sh"
        }

    }
}