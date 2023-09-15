yum install postgresql
$HOME/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres
$HOME/.cargo/bin/diesel migration run