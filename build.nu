cargo build --release

cp target/release/artificial_bike_racing build/artificial_bike_racing
cp -r assets build/

docker build -t racing:latest -f dockerfile build/