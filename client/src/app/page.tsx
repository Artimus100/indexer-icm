import { Coins } from "@/components/Coins";
import Link from "next/link";

export default function Home() {
  return (
    <div className="min-h-screen bg-white text-gray-900">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-4">Pump.fun Clone</h1>
          <p className="text-gray-600 mb-6">Discover and trade the latest meme coins</p>
          <div className="flex space-x-4">
            <Link 
              href="/coins"
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Trading Dashboard
            </Link>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-6">
          <Coins />
        </div>
      </div>
    </div>
  );
}
