export async function GET() {
  const data = {
    id: 1,
    name: "foo",
    description: "foo bar bazz",
  };
  return Response.json({ data });
}
