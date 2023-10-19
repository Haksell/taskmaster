# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-22.04"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = "8192"
	  vb.cpus = 8
  end

  config.vm.provision "shell", inline: <<~SHELL
    apt-get update
    apt-get install -y git vim curl build-essential zsh
    
    su -l vagrant -s "/bin/sh" -c "curl https://sh.rustup.rs -sSf | sh -s -- -y"

    su -l vagrant -s "/bin/sh" -c "curl -fsSO https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh; chmod 755 install.sh; ./install.sh --unattended"
    
    chsh -s /bin/zsh vagrant

    echo "cd /vagrant" >> /home/vagrant/.zshrc
  SHELL

end
