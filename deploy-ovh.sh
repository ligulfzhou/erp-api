ssh ovh << EOFOVH
cd /home/debian/erp-api
git pull origin main
/home/debian/.cargo/bin/cargo build -r
/usr/bin/svc restart erp:erp-9100
/usr/bin/svc restart erp:erp-9101
/usr/bin/svc restart erp:erp-9102
/usr/bin/svc restart erp:erp-9103
EOFOVH
