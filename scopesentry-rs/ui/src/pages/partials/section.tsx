export function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="space-y-3">
      <h3 className="text-base font-semibold">{title}</h3>
      <div className="rounded-lg border p-4">{children}</div>
    </section>
  )
}
