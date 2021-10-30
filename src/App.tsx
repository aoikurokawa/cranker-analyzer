import React, { useEffect, useState } from "react";

import Header from "./components/Header";
import Card from "./components/Card";
import Form from "./components/Form";
import { getAllCampaigns } from "./solana";

interface ICard {
  pubId: string;
  name: string;
  description: string;
  amount_donated: number;
  image_link: string;
}

const App = () => {
  const [route, setRoute] = useState(0);
  const [cards, setCards] = useState<any[]>([]);

  useEffect(() => {
    getAllCampaigns().then((val: any) => {
      setCards(val);
      console.log(val);
    });
  }, []);

  return (
    <div className="ui container">
      <Header setRoute={setRoute} />
      {route === 0 ? (
        <div>
          {cards.map((e) => (
            <Card
              key={e.pubId.toString()}
              data={{
                title: e.name,
                description: e.description,
                amount: e.amount_donated.toString(),
                image: e.image_link,
                id: e.pubId,
              }}
              setCards={setCards}
            />
          ))}
        </div>
      ) : (
        <Form
          setRoute={(e) => {
            setRoute(e);
            getAllCampaigns().then((val: any) => {
              setCards(val);
            });
          }}
        />
      )}
    </div>
  );
};

export default App;
