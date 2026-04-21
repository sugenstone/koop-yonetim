#!/usr/bin/env bash
# Koop API smoke test
# Usage:
#   bash tests/api.sh                       # calisan koop-backend @ :3002
#   KOOP_API_URL=http://x:3000 bash tests/api.sh
#
# Cikis kodu: toplam FAIL sayisi (0 = hepsi yesil).
set -u
set -o pipefail

BASE="${KOOP_API_URL:-http://localhost:3002}/api"
EMAIL="${KOOP_TEST_EMAIL:-admin@koop.local}"
SIFRE="${KOOP_TEST_SIFRE:-Busraasya1334.}"

PASS=0; WARN=0; FAIL=0

# ─── Renkler ────────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
  R=$'\e[31m'; G=$'\e[32m'; Y=$'\e[33m'; C=$'\e[36m'; N=$'\e[0m'
else
  R=; G=; Y=; C=; N=
fi

log_pass() { printf "  ${G}[OK %s]${N} %-8s %-38s -> %s\n" "$1" "$2" "$3" "$4"; PASS=$((PASS+1)); }
log_warn() { printf "  ${Y}[-- %s]${N} %-8s %-38s -> %s\n" "$1" "$2" "$3" "$4"; WARN=$((WARN+1)); }
log_fail() { printf "  ${R}[XX %s]${N} %-8s %-38s -> %s\n" "$1" "$2" "$3" "$4"; FAIL=$((FAIL+1)); }

# GET <path> <etiket>  ->  PASS/FAIL kaydi
hit() {
  local method="$1" path="$2" label="$3" expected="${4:-200}"
  local code body
  body=$(curl -s -o /tmp/koop_resp -w "%{http_code}" \
    -X "$method" "$BASE$path" \
    -H "Authorization: Bearer $TOKEN" 2>/dev/null) || { log_fail "000" "$method" "$path" "curl hatasi"; return; }
  code="$body"
  local count
  count=$(python3 -c "import sys,json
try:
  d = json.load(open('/tmp/koop_resp'))
  if isinstance(d, list): print('count=%d' % len(d))
  elif isinstance(d, dict): print('obj')
  else: print('-')
except Exception: print('-')")
  if [[ "$code" == "$expected" ]]; then
    log_pass "$code" "$method" "$path" "$label ($count)"
  else
    log_fail "$code" "$method" "$path" "$label (beklenen=$expected)"
  fi
}

echo
echo "=== KOOP API SMOKE TEST ==="
echo "BASE: $BASE"
echo "USER: $EMAIL"
echo

# ─── 1) Login ───────────────────────────────────────────────────────────────
echo "=== AUTH ==="
TOKEN=$(curl -s -X POST "$BASE/auth/giris" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"sifre\":\"$SIFRE\"}" \
  | python3 -c "import sys,json
try: print(json.load(sys.stdin)['token'])
except: pass")

if [[ -z "$TOKEN" ]]; then
  echo "  ${R}[XX]${N} Login basarisiz - testler iptal"
  exit 1
fi
printf "  ${G}[OK]${N} Login: token uzunluk=%d\n" "${#TOKEN}"

# ─── 2) Kullanici & izin ────────────────────────────────────────────────────
echo
echo "=== KULLANICILAR ==="
hit GET /kullanicilar            "Kullanici listesi"
hit GET /kullanicilar/bekleyenler "Bekleyen kayitlar"

echo
echo "=== IZINLER ==="
hit GET /izinler                 "Tum izinler"
hit GET /izinler/benim           "Benim izinlerim"
hit GET /izinler/roller/admin    "Admin rol izinleri"

# ─── 3) Kasa ───────────────────────────────────────────────────────────────
echo
echo "=== KASALAR ==="
hit GET /kasalar                 "Kasa listesi"
# Dinamik ID secimi
KASA_ID=$(curl -s "$BASE/kasalar" -H "Authorization: Bearer $TOKEN" \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print(d[0]['id'] if d else '')")
if [[ -n "$KASA_ID" ]]; then
  hit GET "/kasalar/$KASA_ID"              "Kasa detay"
  hit GET "/kasalar/$KASA_ID/hareketler"   "Kasa hareketleri"
  hit GET "/kasalar/$KASA_ID/transferler"  "Kasa transferleri"
fi

# ─── 4) Hissedar ───────────────────────────────────────────────────────────
echo
echo "=== HISSEDARLAR ==="
hit GET /hissedarlar             "Hissedar listesi"
HSD_ID=$(curl -s "$BASE/hissedarlar" -H "Authorization: Bearer $TOKEN" \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print(d[0]['id'] if d else '')")
if [[ -n "$HSD_ID" ]]; then
  hit GET "/hissedarlar/$HSD_ID"           "Hissedar detay"
  hit GET "/hissedarlar/$HSD_ID/cuzdan"    "Hissedar cuzdan"
  hit GET "/hissedarlar/$HSD_ID/atamalar"  "Hissedar atamalar"
  hit GET "/hissedarlar/$HSD_ID/borclar"   "Hissedar borclar"
fi

# ─── 5) Donem ──────────────────────────────────────────────────────────────
echo
echo "=== DONEMLER ==="
hit GET /donemler                "Donem listesi"
DNM_ID=$(curl -s "$BASE/donemler" -H "Authorization: Bearer $TOKEN" \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print(d[0]['id'] if d else '')")
if [[ -n "$DNM_ID" ]]; then
  hit GET "/donemler/$DNM_ID"              "Donem detay"
  hit GET "/donemler/$DNM_ID/toplantilar"  "Donem toplantilari"
  hit GET "/donemler/$DNM_ID/borclar"      "Donem borclari"
fi

# ─── 6) Hisse ──────────────────────────────────────────────────────────────
echo
echo "=== HISSELER ==="
hit GET /hisseler                "Hisse listesi"
HS_ID=$(curl -s "$BASE/hisseler" -H "Authorization: Bearer $TOKEN" \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print(d[0]['id'] if d else '')")
if [[ -n "$HS_ID" ]]; then
  hit GET "/hisseler/$HS_ID"               "Hisse detay"
  hit GET "/hisseler/$HS_ID/atamalar"      "Hisse atamalari"
  hit GET "/hisseler/$HS_ID/borclar"       "Hisse borclari"
  hit GET "/hisseler/$HS_ID/satis"         "Hisse aktif satis"
fi

# ─── 7) Gelir/Gider ────────────────────────────────────────────────────────
echo
echo "=== GELIR/GIDER ==="
hit GET /gelir-gider/kategoriler "Kategoriler"
hit GET /gelir-gider/kayitlar    "Kayitlar"

# ─── Sonuc ─────────────────────────────────────────────────────────────────
echo
echo "============================="
printf "  PASS: ${G}%d${N}   WARN: ${Y}%d${N}   FAIL: ${R}%d${N}\n" "$PASS" "$WARN" "$FAIL"
echo "============================="

exit "$FAIL"
