export function FormRow({ label, control }: { label: string; control: React.ReactNode }) {
  return (
    <div className="grid gap-2">
      <div className="text-sm text-muted-foreground">{label}</div>
      {control}
    </div>
  )
}
