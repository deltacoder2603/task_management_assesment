#!/bin/bash

set -e

BASE_URL="http://localhost:3000"

echo "========================================="
echo "1. Seed Users"
echo "========================================="

curl -s -X POST \
"$BASE_URL/seed/users"

echo ""
echo ""

echo "========================================="
echo "2. Admin Login"
echo "========================================="

ADMIN_LOGIN=$(curl -s -X POST \
"$BASE_URL/auth/login" \
-H "Content-Type: application/json" \
-d '{
  "email":"admin@example.com",
  "password":"admin123"
}')

echo "$ADMIN_LOGIN"

ADMIN_CHALLENGE_ID=$(echo "$ADMIN_LOGIN" | jq -r '.login_challenge_id')

echo ""
echo "Admin Challenge ID:"
echo "$ADMIN_CHALLENGE_ID"

echo ""
echo "========================================="
echo "3. Fetch Admin 2FA Code"
echo "========================================="

ADMIN_CODE=$(curl -s \
"$BASE_URL/dev/email-logs/latest" \
| jq -r '.code')

echo "Admin Code: $ADMIN_CODE"

echo ""
echo "========================================="
echo "4. Verify Admin 2FA"
echo "========================================="

ADMIN_VERIFY=$(curl -s -X POST \
"$BASE_URL/auth/verify-2fa" \
-H "Content-Type: application/json" \
-d "{
  \"challenge_id\":\"$ADMIN_CHALLENGE_ID\",
  \"code\":\"$ADMIN_CODE\"
}")

echo "$ADMIN_VERIFY"

ADMIN_TOKEN=$(echo "$ADMIN_VERIFY" | jq -r '.access_token')

echo ""
echo "Admin JWT:"
echo "$ADMIN_TOKEN"

echo ""
echo "========================================="
echo "5. Create 5 Tasks"
echo "========================================="

TASK_IDS=()

for i in 1 2 3 4 5
do

RESULT=$(curl -s -X POST \
"$BASE_URL/tasks" \
-H "Authorization: Bearer $ADMIN_TOKEN" \
-H "Content-Type: application/json" \
-d "{
  \"title\":\"Task $i\",
  \"description\":\"Description $i\",
  \"priority\":\"high\"
}")

echo "$RESULT"

TASK_ID=$(echo "$RESULT" | jq -r '.task_id')

TASK_IDS+=("$TASK_ID")

done

echo ""
echo "Created Tasks:"
printf '%s\n' "${TASK_IDS[@]}"

echo ""
echo "========================================="
echo "6. Assign First 3 Tasks To James Bond"
echo "========================================="

ASSIGN_RESPONSE=$(curl -s -X POST \
"$BASE_URL/tasks/assign" \
-H "Authorization: Bearer $ADMIN_TOKEN" \
-H "Content-Type: application/json" \
-d "{
  \"assigned_to_email\":\"jamesbond@example.com\",
  \"task_ids\": [
    \"${TASK_IDS[0]}\",
    \"${TASK_IDS[1]}\",
    \"${TASK_IDS[2]}\"
  ]
}")

echo "$ASSIGN_RESPONSE"

echo ""
echo "========================================="
echo "7. James Bond Login"
echo "========================================="

JAMES_LOGIN=$(curl -s -X POST \
"$BASE_URL/auth/login" \
-H "Content-Type: application/json" \
-d '{
  "email":"jamesbond@example.com",
  "password":"bond007"
}')

echo "$JAMES_LOGIN"

JAMES_CHALLENGE_ID=$(echo "$JAMES_LOGIN" | jq -r '.login_challenge_id')

echo ""
echo "James Challenge ID:"
echo "$JAMES_CHALLENGE_ID"

echo ""
echo "========================================="
echo "8. Fetch James 2FA Code"
echo "========================================="

JAMES_CODE=$(curl -s \
"$BASE_URL/dev/email-logs/latest" \
| jq -r '.code')

echo "James Code: $JAMES_CODE"

echo ""
echo "========================================="
echo "9. Verify James 2FA"
echo "========================================="

JAMES_VERIFY=$(curl -s -X POST \
"$BASE_URL/auth/verify-2fa" \
-H "Content-Type: application/json" \
-d "{
  \"challenge_id\":\"$JAMES_CHALLENGE_ID\",
  \"code\":\"$JAMES_CODE\"
}")

echo "$JAMES_VERIFY"

JAMES_TOKEN=$(echo "$JAMES_VERIFY" | jq -r '.access_token')

echo ""
echo "James JWT:"
echo "$JAMES_TOKEN"

echo ""
echo "========================================="
echo "10. James Tries To Create Task"
echo "========================================="

curl -i -X POST \
"$BASE_URL/tasks" \
-H "Authorization: Bearer $JAMES_TOKEN" \
-H "Content-Type: application/json" \
-d '{
  "title":"Forbidden Task",
  "description":"Should fail",
  "priority":"low"
}'

echo ""
echo ""
echo "Expected: HTTP 403"

echo ""
echo "========================================="
echo "11. First Call (Database)"
echo "========================================="

FIRST=$(curl -s \
"$BASE_URL/tasks/view-my-tasks" \
-H "Authorization: Bearer $JAMES_TOKEN")

echo "$FIRST" | jq

echo ""
echo "Cache should be:"
echo "false"

echo ""
echo "========================================="
echo "12. Second Call (Cache)"
echo "========================================="

SECOND=$(curl -s \
"$BASE_URL/tasks/view-my-tasks" \
-H "Authorization: Bearer $JAMES_TOKEN")

echo "$SECOND" | jq

echo ""
echo "Cache should be:"
echo "true"

echo ""
echo "========================================="
echo "VALIDATION COMPLETE"
echo "========================================="