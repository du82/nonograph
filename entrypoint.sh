#!/bin/sh

# Create content directory if it doesn't exist, then fix permissions
mkdir -p /app/content
chown -R nonograph:nonograph /app/content

mkdir -p /var/lib/tor/hidden_service
chown -R debian-tor:debian-tor /var/lib/tor/hidden_service
chmod 700 /var/lib/tor/hidden_service

# Start Tor as debian-tor user
sudo -u debian-tor tor -f /etc/tor/torrc &

# Wait for the .onion hostname file to appear
echo "Waiting for Tor hidden service to be ready..."
ONION_FILE="/var/lib/tor/hidden_service/hostname"
i=0
while [ ! -f "$ONION_FILE" ]; do
    sleep 1
    i=$((i + 1))
    if [ $i -ge 60 ]; then
        echo "Tor hidden service did not start within 60 seconds, check logs above."
        break
    fi
done

if [ -f "$ONION_FILE" ]; then
    ONION=$(cat "$ONION_FILE" 2>/dev/null)
    echo ""
    echo "========================================="
    echo "  Your .onion address:"
    echo "  http://$ONION"
    echo "========================================="
    echo ""
fi

# Drop to nonograph user and launch the app
exec su -s /bin/sh -c 'exec /app/nonograph' nonograph
