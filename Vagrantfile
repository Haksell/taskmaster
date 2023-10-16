# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-22.04"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = "4096"
	vb.cpus = 6
  end

  config.vm.provision "shell", inline: <<~SHELL
    apt-get update
    apt-get install -y git vim curl
    su vagrant <<EOF
    curl https://sh.rustup.rs -sSf | sh -s -- -y;
    EOF
  SHELL

end
