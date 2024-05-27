GET http://localhost:8080
HTTP 404

GET http://localhost:8080/
HTTP 404

GET http://localhost:8080/config.json
HTTP 200
Content-Type: application/json
{"myKey":"value2"}
