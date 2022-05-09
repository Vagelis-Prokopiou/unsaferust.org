import type {Component} from 'solid-js';

import styles from './App.module.css';
import ProjectStatsList from './components/ProjectStatsList';
import ProjectStatDetails from './components/ProjectStatDetails';
import Pager from './components/Pager';
import Header from './components/Header';
import Footer from './components/Footer';
import Home from './components/Home';
import {JSX, createResource, Show, createSignal, createEffect} from "solid-js";

// Constants
const ROUTE_HOME = 'home';
const ROUTE_LIST = 'list';
const ROUTE_DETAILS = 'details';


// Signals
const [getRoute, setRoute] = createSignal(ROUTE_HOME);
const [getProjectId, setProjectId] = createSignal(0);
const [getPaginationOptions, setPaginationOptions] = createSignal({
    id: 0,
    limit: 25,
    direction: "asc"
});

// Resources
const [projectStats] = createResource(
    getProjectId,
    async (projectId) => {
        if (projectId === 0) return [];
        const _ = await fetch(`${import.meta.env.VITE_SERVER_URL}/project-stats/${projectId}`);
        const name = projectStatsList().project_stats.filter(item => item.project_id === projectId)[0].name;
        const data = (await _.json()).map(item => ({...item, name}));
        return data;
    });

const [projectStatsList] = createResource(
    getPaginationOptions,
    async (paginationOptions) => {
        const {id, limit, direction} = paginationOptions;
        const url = `${import.meta.env.VITE_SERVER_URL}/project-stats?id=${id}&limit=${limit}&direction=${direction}`;
        const _ = await fetch(url);
        const data = await _.json();
        return data;
    });

function getMaxProjectId() {
    return projectStatsList()
        .project_stats
        .map(item => item.project_id)
        .sort()
        .reverse()[0] || 0;
}

function getMinProjectId() {
    return projectStatsList()
        .project_stats
        .map(item => item.project_id)
        .sort()[0] || 0;
}

const App: () => JSX.Element = () => {
    return (
        <div id="app" class="flex flex-col min-h-screen font-roboto dark:bg-gray-900">
            <Header
                navigate={(route) => setRoute(route)}
                ROUTE_HOME={ROUTE_HOME}
                ROUTE_LIST={ROUTE_LIST}
            />

            <main class="flex-1 lg:mt-20">
                <section class="container px-4 py-10 mx-auto">

                    <Show when={getRoute() === ROUTE_HOME}>
                        <Home/>
                    </Show>

                    <Show when={getRoute() === ROUTE_LIST}>
                        <ProjectStatsList
                            data={!projectStatsList.loading && projectStatsList().project_stats}
                            total={!projectStatsList.loading && projectStatsList().meta.total}
                            navigate={(id) => {
                                // Load the resource.
                                setProjectId(id);
                                // Set the route.
                                setRoute(ROUTE_DETAILS);
                            }}
                            getPaginationOptions={getPaginationOptions}
                            setPaginationOptions={setPaginationOptions}
                        />

                        <Pager
                            numberOfCurrentItems={!projectStatsList.loading && projectStatsList().project_stats.length}
                            numberOfTotalItems={!projectStatsList.loading && projectStatsList().meta.total}
                            paginate={(direction) => {
                                // This is for desc
                                let minProjectId = getMinProjectId();
                                const itemsPerPage = parseInt(document.getElementById("items-per-page").value);
                                if (minProjectId < itemsPerPage) {
                                    minProjectId = itemsPerPage;
                                }
                                const newOptions = {
                                    direction,
                                    limit: getPaginationOptions().limit,
                                    id: (direction === 'asc')
                                        ? getMaxProjectId()
                                        : minProjectId
                                };
                                const existingOptions = getPaginationOptions();
                                if (JSON.stringify(newOptions) === JSON.stringify(existingOptions)) {
                                    return false;
                                }
                                setPaginationOptions({
                                    direction,
                                    limit: getPaginationOptions().limit,
                                    id: (direction === 'asc')
                                        ? getMaxProjectId()
                                        : minProjectId
                                });
                            }}
                        />
                    </Show>

                    <Show when={!projectStats.loading && getRoute() === ROUTE_DETAILS}>
                        <ProjectStatDetails
                            data={projectStats()}
                            navigate={() => setRoute(ROUTE_LIST)}
                        />
                    </Show>
                </section>
            </main>

            <Footer/>
        </div>
    );
};

export default App;
