variable "prod_user" {
  type      = string
  sensitive = true
}

variable "prod_host" {
  type      = string
  sensitive = true
}

variable "prod_pass" {
  type      = string
  sensitive = true
}

variable "prod_dir" {
  default = "/home/ubuntu/prod/"
}

variable "repo_name" {
  default = "nervo_bot"
}

// Provision production with essential scripts to bootstrap the server (run on github actions)
resource "null_resource" "bootstrap" {

  triggers = {
    always_trigger = timestamp()
  }

  provisioner "remote-exec" {
    connection {
      type     = "ssh"
      user     = var.prod_user
      password = var.prod_pass
      host     = var.prod_host
    }

    inline = [
      "cd ${var.prod_dir}/${var.repo_name}",
      "git pull",
      "cd ${var.prod_dir}/${var.repo_name}/infra/deployment",
      "devbox run task k3d_cluster_deploy"
    ]
  }
}