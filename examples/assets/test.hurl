GET http://localhost:8080
HTTP 200
Content-Type: text/html; charset=utf-8
[Captures]
root_body: body
[Asserts]
file,root/index.html;

GET http://localhost:8080/
HTTP 200
Content-Type: text/html; charset=utf-8
[Asserts]
body == {{root_body}}

GET http://localhost:8080/styles.css
HTTP 200
Content-Type: text/css
[Asserts]
file,root/styles.css;
