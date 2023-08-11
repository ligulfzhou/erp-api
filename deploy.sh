ssh hosthatch << EOFHOSTHATCH
ssh ovh << EOFOVH
cd /home/debian/erp-api
/home/debian/.cargo/bin/cargo build -r
/usr/bin/svc restart erp-9100
EOFOVH
EOFHOSTHATCH