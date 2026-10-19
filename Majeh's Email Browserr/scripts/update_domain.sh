#!/usr/bin/env bash
# Point the whole project at a new domain.
#
#   ./scripts/update_domain.sh yourdomain.com
#
# Rewrites every hardcoded occurrence of the current domain so the homepage
# link, the in-app links, the canonical tags, robots.txt, the sitemap and the
# OAuth consent screen all stay in sync. Google requires the homepage privacy
# link to match the URL on the consent screen exactly.
set -euo pipefail

NEW="${1:-}"
if [ -z "$NEW" ]; then
  echo "usage: $0 <new-domain.com>" >&2
  exit 1
fi
NEW="${NEW#https://}"
NEW="${NEW%%/*}"

cd "$(dirname "$0")/.."

FILES=$(grep -rl "majehsemailviewer\.web\.app" \
  --include='*.html' --include='*.rs' --include='*.md' --include='*.txt' --include='*.xml' \
  --include='*.json' . || true)

if [ -z "$FILES" ]; then
  echo "No occurrences of the old domain found - nothing to do."
  exit 0
fi

for f in $FILES; do
  sed -i "s|majehsemailviewer\.web\.app|$NEW|g" "$f"
  echo "updated: $f"
done

cat << 'REMINDER'

Next steps (not automated - do these by hand):
  1. Firebase console -> Project settings -> Add custom domain -> $NEW
     (then add the A/AAAA/CNAME DNS records at your registrar)
  2. Google Cloud -> APIs & Services -> Credentials -> your OAuth client:
       Authorized JavaScript origins : https://$NEW
       Authorized redirect URIs     : https://$NEW/auth/callback/google
  3. Google Cloud -> OAuth consent screen -> App information:
       Homepage          : https://$NEW
       Privacy policy    : https://$NEW/privacy.html
  4. Verify you own $NEW (Search Console -> Add property -> Domain ->
     DNS TXT record, or the HTML file upload method)
  5. Wait 24h after ownership is verified, then resubmit for verification.
REMINDER
