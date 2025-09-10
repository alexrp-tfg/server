time seq 2000 | xargs -P 2000 -n 1 -I {} \
  curl -s -o /dev/null \
  -X POST \
  --header 'authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3MjhjNWJlOC04ZTNmLTRiMDQtYmQzYy0zOTQ1ZGRhMzZhMGQiLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZSI6IkFkbWluIiwiZXhwIjoxNzU3MjcyODI4fQ.oOJlkd0s7oWehB2uyXbDaFv7_XngtivqOLYMqn4JFb0' \
  --header 'content-type: multipart/form-data' \
  --header 'x-file-size: 1188' \
  --form 'file=@/home/alexrp/Pictures/ddybdlp-bf1b185d-687d-4f99-9e39-9dc839f28148.gif;type=image/gif' \
  --url "http://localhost:8000/api/media/upload"
