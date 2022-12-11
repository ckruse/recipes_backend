# Rezepte-Datenbank: Backend

## Installation

Download the asset from the [latest release](https://github.com/ckruse/recipes_backend/releases/latest) or compile the source yourself:

```
git clone https://github.com/ckruse/recipes_backend.git
cd recipes_backend
cargo build --release
```

## Configure

The backend is configured by environment variables. The following variables are recognized:

```
LISTEN=127.0.0.1:8080
DATABASE_URL=postgresql://localhost/recipes_dev
COOKIE_KEY=some_random_hool9ioXahT6aiheeyiphao8koothubahthoweiD
PICTURE_DIR=/path/to/pictures/storage/dir
AVATAR_DIR=/path/to/avatars/storage/dir
```

## Run

Just run the binary:

```
./recipes
```
