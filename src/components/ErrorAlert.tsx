import { WarningCircle } from "@phosphor-icons/react";

export function ErrorAlert({ message }: { message: string }) {
  return (
    <div className="alert" role="alert">
      <WarningCircle size={16} />
      <p>{message}</p>
    </div>
  );
}
