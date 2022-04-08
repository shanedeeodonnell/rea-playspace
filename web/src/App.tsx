import React, { useEffect, useState } from "react";
import "@shoelace-style/shoelace/dist/themes/light.css";
import { setBasePath } from "@shoelace-style/shoelace/dist/utilities/base-path";
import { BrowserRouter, Link, Route, Routes } from "react-router-dom";
import "./App.css";
//import EconomicResourceList from "./EconomicResourceList";
//import CreateEconomicResource from "./CreateEconomicResource";
import Header from "./Header";
import LeftScreenNavMenu from "./LeftScreenNavMenu";
//import { SlButton } from "@shoelace-style/shoelace/dist/react";
import Resources from "./routes/Resources";
import NewResource from "./routes/NewResource";
import ResourceTransfer from "./routes/ResourceTransfer";
import FlowLayout from "./routes/FlowLayout";
import { APP_ID, ZOME_NAME, ADMIN_PORT, APP_PORT } from './holochainConf';
import { AdminWebsocket, AppWebsocket, HoloHash, CellId } from '@holochain/client';
import { app } from "electron";
const ADMIN_WS_URL = `ws://localhost:${ADMIN_PORT}`;
const APP_WS_URL = `ws://localhost:${APP_PORT}`;

setBasePath(
  "https://cdn.jsdelivr.net/npm/@shoelace-style/shoelace@2.0.0-beta.70/dist/"
);

interface Props {}

function HashToString(buff: Uint8Array): string {
  return buff.reduce((prev: string, curr: number) => {
    return prev + curr.toString(16).padStart(2, '0');
  }, '');
}

function StringToHash(s: string): Uint8Array {
  const b = new Uint8Array(Math.ceil(s.length/2));
  for (let i = 0; i < b.byteLength; i++) {
    b[i] = parseInt(s.slice(i*2,(i*2)+2),16);
  }
  return b;
}

const App: React.FC<Props> = () => {
  const [myAgentId, setMyAgentId] = useState<string>();
  const [client, setClient] = useState<AppWebsocket>();

  const connect = async () => {
    const adminWs = await AdminWebsocket.connect(ADMIN_WS_URL);
    console.log('admin websocket opened');
    const appWs = await AppWebsocket.connect(APP_WS_URL);
    console.log('app websocket opened');
    adminWs.client.socket.addEventListener('close', () => {
      console.log('admin websocket closed');
    });
    appWs.client.socket.addEventListener('close', () => {
      console.log('app websocket closed');
    })

    if (adminWs.client.socket.readyState === adminWs.client.socket.OPEN
      && appWs.client.socket.readyState === appWs.client.socket.OPEN) {
      const appIds = await adminWs.listActiveApps();
      const infos = await Promise.all(
        appIds.map(async installed_app_id => {
          const appInfo = await appWs.appInfo({ installed_app_id });
          return {
            ...appInfo,
            cellIdString: HashToString(appInfo.cell_data[0].cell_id[0]) + ':' + HashToString(appInfo.cell_data[0].cell_id[1])
          };
        })
      );
      console.log(infos);
      const cellIds = await adminWs.listCellIds();
      console.log (cellIds);

      const appDnas = await adminWs.listDnas();
      console.log(appDnas);
      
    }
    
    //const zome_res = await client.callZome()
    
    // @ts-ignore
    setClient(appWs);
    setMyAgentId('hello');
  };

  useEffect(() => {
    connect();
  }, []);

  if (!client) {
    return <div>Making websocket connection...</div>;
  }

  return (
    <BrowserRouter>
      <div className="container">
        <Header name={myAgentId} />
        <div className="below-header">
          <LeftScreenNavMenu />

          <div className="main-panel">
              <Routes>
                <Route
                  path="/flow"
                  element={<FlowLayout myAgentId={myAgentId}/>}
                />
                <Route
                  path="/resources"
                  element={<Resources myAgentId={myAgentId} />}
                />
                <Route
                  path="/resources/transfer"
                  element={<ResourceTransfer myAgentId={myAgentId} />}
                />
                <Route
                  path="/resources/new"
                  element={<NewResource myAgentId={myAgentId} />}
                />
              </Routes>
          </div>
        </div>
      </div>
    </BrowserRouter>
  );
};

export default App;