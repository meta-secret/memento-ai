import .common

apiVersion = "v1"
kind = "Service"
metadata = {
    name = "${common.projectName}-service"
}
spec = {
    selector = {
        "app.kubernetes.io/name" = common.projectName
    }
    ports = [
        {
            name = "${common.projectName}-service-port"
            $protocol = "TCP"
            port = 3001
            targetPort = 3000
        }
    ]
}
