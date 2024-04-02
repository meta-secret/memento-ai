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
      "mkdir -p ${var.prod_dir}",
      "rm -rf ${var.prod_dir}/provision",
    ]
  }

  provisioner "file" {
    source      = "../provision"
    destination = var.prod_dir

    connection {
      type     = "ssh"
      user     = var.prod_user
      password = var.prod_pass
      host     = var.prod_host
    }
  }

  provisioner "remote-exec" {
    connection {
      type     = "ssh"
      user     = var.prod_user
      password = var.prod_pass
      host     = var.prod_host
    }

    inline = [
      "cd ${var.prod_dir}/provision",
      "chmod +x ${var.prod_dir}/provision/nixw.sh",
      "./nixw.sh",
      ". /home/ubuntu/.nix-profile/etc/profile.d/nix.sh",
      "nix-shell --run 'just provision'"
    ]
  }
}