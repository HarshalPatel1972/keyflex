/** "Ctrl + J" drawn as keycaps. */
export function Keys({ keys }: { keys: string }) {
  return (
    <span className="keys">
      {keys.split(" + ").map((key) => (
        <kbd key={key}>{key}</kbd>
      ))}
    </span>
  );
}
