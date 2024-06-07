tasks.register("imgName") {
    val infra = findProperty("infra") as String
    val registry = findProperty("registry") as String
    val repository = findProperty("repository") as String
    val k8sAppName = findProperty("k8sAppName") as String
    val appVersion = findProperty("appVersion") as String

    doLast {
        val baseImageName = "${registry}/${repository}:${k8sAppName}"
        if (infra == "prod") {
            println("${baseImageName}_v${appVersion}")
            return@doLast
        }

        if (infra == "dev") {
            println("${baseImageName}_${userName()}_dev_v${appVersion}")
            return@doLast
        }

        throw IllegalStateException("Unknown INFRA parameter: $infra")
    }
}

fun userName(): String = System.getenv("USER")