cargo build --release

cp target/release/artificial_bike_racing build/artificial_bike_racing
cp -r assets build/
cp -r proto build/

docker build -t julianbieber/artificial-bike-racing:latest -f dockerfile build/