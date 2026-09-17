import * as http from 'node:http'

const server = http.createServer((req, res) => {
  const url = new URL(req.url ?? '/', `http://${req.headers.host ?? 'localhost'}`)

  const pathName = decodeURIComponent(url.pathname)

  res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' })
  res.end(pathName)
})

const port = Number(process.env.PORT ?? 3000)

server.listen(port, () => {
  console.log(`Server listening on http://localhost:${port}`)
})