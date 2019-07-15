#!/usr/bin/env bash

curl -X POST \
  http://10.0.0.50:3000/api/user/06502246420/cc/AT
  
curl -X GET \
  http://10.0.0.50:3000/api/user/06502246420/cc/AT
 
curl -X GET \
  http://10.0.0.50:3000/api/user/06502246420/cc/AT

curl -X PUT \
  http://10.0.0.50:3000/api/user/06502246420/cc/AT/led/true

curl -X PUT \
  http://10.0.0.50:3000/api/user/06502246420/cc/AT/led/false

curl -X PUT \
  http://10.0.0.50:3000/api/user/06502246420/cc/AT/ \
  -H 'Accept: */*' \
  -H 'Cache-Control: no-cache' \
  -H 'Connection: keep-alive' \
  -H 'Content-Type: application/json' \
  -H 'Host: 10.0.0.50:3000' \
  -H 'accept-encoding: gzip, deflate' \
  -H 'cache-control: no-cache' \
  -H 'content-length: 40' \
  -d '{
	"description": "Das ist ein Update"
}'
