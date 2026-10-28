#!/usr/bin/env bash
# Creates a self-signed code-signing certificate named "Lock In Local" in your login keychain.
#
# Signing every build with the same certificate keeps macOS privacy permissions (Screen Recording)
# attached to the app across rebuilds. Run once; macOS may ask for your keychain password.
set -euo pipefail

NAME="Lock In Local"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
# macOS's bundled LibreSSL writes PKCS#12 files that `security import` can read.
OPENSSL=/usr/bin/openssl

if security find-certificate -c "$NAME" "$KEYCHAIN" >/dev/null 2>&1; then
  echo "\"$NAME\" already exists in your login keychain."
  exit 0
fi

DIR=$(mktemp -d)
trap 'rm -rf "$DIR"' EXIT

cat > "$DIR/cert.cnf" <<EOF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $NAME
[ext]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
EOF

"$OPENSSL" req -x509 -newkey rsa:2048 -nodes -days 3650 -config "$DIR/cert.cnf" \
  -keyout "$DIR/key.pem" -out "$DIR/cert.pem" 2>/dev/null
"$OPENSSL" pkcs12 -export -inkey "$DIR/key.pem" -in "$DIR/cert.pem" -name "$NAME" \
  -out "$DIR/cert.p12" -passout pass:lockin

# -T lets codesign use the key without asking each time.
security import "$DIR/cert.p12" -k "$KEYCHAIN" -P lockin -T /usr/bin/codesign

echo "Created \"$NAME\". Build with: pnpm build:app"
