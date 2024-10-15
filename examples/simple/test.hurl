GET http://localhost:8080
HTTP 200
Content-Type: text/html
[Captures]
root_body: body
[Asserts]
file,root/index.html;

GET http://localhost:8080/
HTTP 200
Content-Type: text/html
[Asserts]
body == {{root_body}}

GET http://localhost:8080/my/spa/route
HTTP 200
Content-Type: text/html
[Asserts]
body == {{root_body}}
