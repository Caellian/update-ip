default:
    @just --list

bin := "update-ip"
service_user := "update-ip"

build *args:
    cargo build --release {{args}}

install-systemd *args: (build args)
    #!/bin/sh
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin {{service_user}} || true
    sudo install -m 755 target/release/{{bin}} /usr/local/bin/{{bin}}
    sudo install -m 644 dist/systemd/{{bin}}.service /etc/systemd/system/
    sudo install -m 644 dist/systemd/{{bin}}.timer /etc/systemd/system/
    sudo systemctl daemon-reload
    echo "Edit /etc/systemd/system/{{bin}}.service with your environment variables, then run:"
    echo "  sudo systemctl enable --now {{bin}}.timer"

install-openrc *args: (build args)
    #!/bin/sh
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin {{service_user}} || true
    sudo install -m 755 target/release/{{bin}} /usr/local/bin/{{bin}}
    sudo install -m 755 dist/openrc/{{bin}} /etc/init.d/{{bin}}
    sudo install -m 644 dist/openrc/{{bin}}.conf /etc/conf.d/{{bin}}
    echo "Edit /etc/conf.d/{{bin}} with your environment variables, then run:"
    echo "  sudo rc-update add {{bin}} default"
    echo "OpenRC has no timers — install the crontab for periodic runs:"
    echo "  sudo crontab -u {{service_user}} dist/crontab"

install-runit *args: (build args)
    #!/bin/sh
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin {{service_user}} || true
    sudo install -m 755 target/release/{{bin}} /usr/local/bin/{{bin}}
    echo "Edit dist/crontab with your environment variables, then install:"
    echo "  sudo crontab -u {{service_user}} dist/crontab"

alias install := install-systemd

uninstall-systemd:
    #!/bin/sh
    sudo systemctl disable --now {{bin}}.timer || true
    sudo systemctl disable --now {{bin}}.service || true
    sudo rm -f /etc/systemd/system/{{bin}}.service /etc/systemd/system/{{bin}}.timer
    sudo systemctl daemon-reload
    sudo rm -f /usr/local/bin/{{bin}}
    sudo userdel {{service_user}} || true

uninstall-openrc:
    #!/bin/sh
    sudo rc-update del {{bin}} default || true
    sudo crontab -r -u {{service_user}} || true
    sudo rm -f /etc/init.d/{{bin}} /etc/conf.d/{{bin}}
    sudo rm -f /usr/local/bin/{{bin}}
    sudo userdel {{service_user}} || true

uninstall-runit:
    #!/bin/sh
    sudo crontab -r -u {{service_user}} || true
    sudo rm -f /usr/local/bin/{{bin}}
    sudo userdel {{service_user}} || true

alias uninstall := uninstall-systemd
