#!/bin/bash

[ "$UID" -ne 0 ] && echo "You should run this script as a root " && exit 1
[ -x dist/web ] || [ -x dist/harvester ] || echo "Executables not found in dist/" && exit 1

apt-get update
apt-get install -y vim bluetooth bluez bluez-tools pi-bluetooth

mkdir -p /opt/pibq

cp -R web migrations /opt/pibq
cp dist/web dist/harvester /opt/pibq

chown -R pi opt/pibq

cp pi_config/bluetooth_rfcomm.service /lib/systemd/system/
cp pi_config/pibq_web.service /lib/systemd/system/
cp pi_config/pibq_harvester.service /lib/systemd/system/

cp pi_config/pibq.env /etc/default/pibq

systemctl enable bluetooth_rfcomm
systemctl enable pibq_harvester
systemctl enable pibq_web

systemctl start pibq_web
