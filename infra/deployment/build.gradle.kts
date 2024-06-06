
fun k8sClusterName () = System.getenv("USER")

tasks.register("user") {
    doLast {
        println(k8sClusterName() )
    }
}
