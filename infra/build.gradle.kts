
object TaskNames {
    val qwa = "qwa"
    val hi = "hi"
}

tasks.register(TaskNames.qwa) {
    doLast {
        println("qwa")
    }
}

tasks.register(TaskNames.hi) {
    dependsOn(TaskNames.qwa)
    doLast {
        println("hi")
    }
}
