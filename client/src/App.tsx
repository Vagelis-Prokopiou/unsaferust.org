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
const [getPaginationOptions, setPaginationOptions] = createSignal({limit: 25, page: 1, name: ""});

// Resources
const [projectStats] = createResource(
    getProjectId,
    async (projectId) => {
        if (projectId === 0) return [];
        const _ = await fetch(`${import.meta.env.VITE_SERVER_URL}/project-stats/${projectId}`);
        const name = projectStatsList().projectStats.filter(item => item.project_id === projectId)[0].name;
        const data = (await _.json()).map(item => ({...item, name}));
        return data;
    });

const [projectStatsList] = createResource(
    getPaginationOptions,
    async (paginationOptions) => {
        const {page, limit, name} = paginationOptions;
        let uri = `project-stats?page=${page}&limit=${limit}`;
        if (name) {
            uri += `&name=${name}`;
        }
        const url = `${import.meta.env.VITE_SERVER_URL}/${uri}`;
        const _ = await fetch(url);
        const data = await _.json();
        data.projectStats = data.projectStats.map(i => {
            const obj = {...i};
            const percentage = obj.unsafe_lines > 0 ? (obj.unsafe_lines / obj.code_lines) : 0;
            obj.percentage = `${percentage.toFixed(3)}%`;
            return obj;
        });
        return data;
    });

/* ================== */
/* Utility functions  */
/* ================== */
const getNumberOfShownRecords = function () {
    const paginationOptions = getPaginationOptions();
    return (paginationOptions.page - 1) * paginationOptions.limit + projectStatsList().projectStats.length;
}

/* ================== */
/* Google chart stuff */
/* ================== */
const [getChart, setChart] = createSignal(null);
createEffect(() => {
    const chart = getChart();
    chart && chart();
});
google.charts.load('current', {packages: ['corechart', 'line']});

function createChart(data) {
    if (!data) {
        throw 'no data provided to createChart';
    }
    if (data.length < 2) {
        return false;
    }
    const rows = data.map(r => [r.created_at, r.unsafe_lines]).reverse();
    const dt = new google.visualization.DataTable();
    dt.addColumn('string', 'Date');
    dt.addColumn('number', 'Unsafe lines');
    dt.addRows(rows);
    const options = {
        hAxis: {title: 'Date'},
        vAxis: {
            title: 'Number of lines',
            format: 'decimal'
        },
        width: '100%'
    };
    const targetElement = document.getElementById('chart_div');
    var chart = new google.charts.Line(targetElement);
    return () => chart.draw(dt, google.charts.Line.convertOptions(options));
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
                            data={!projectStatsList.loading && projectStatsList().projectStats}
                            total={!projectStatsList.loading && projectStatsList().meta.total}
                            navigate={(id) => {
                                // Load the resource.
                                setProjectId(id);
                                // Set the route.
                                setRoute(ROUTE_DETAILS);
                            }}
                            getPaginationOptions={getPaginationOptions}
                            setPaginationOptions={setPaginationOptions}
                            search={(crateName) => {
                                const paginationOptions = {...getPaginationOptions()};
                                if (paginationOptions.name === crateName) return false;
                                paginationOptions.name = crateName;
                                paginationOptions.page = 1;
                                setPaginationOptions(paginationOptions);
                            }}
                        />

                        <Pager
                            numberOfShownRecords={getNumberOfShownRecords}
                            numberOfTotalItems={!projectStatsList.loading && projectStatsList().meta.total}
                            paginate={(direction) => {
                                // This is for desc
                                const newOptions = {...getPaginationOptions()};
                                if (direction === 'asc') {
                                    const total = projectStatsList().meta.total;
                                    const shown = getNumberOfShownRecords();
                                    if (shown >= total) return;
                                    newOptions.page += 1;
                                } else {
                                    if (newOptions.page < 2) return;
                                    newOptions.page -= 1;
                                }
                                setPaginationOptions(newOptions);
                            }}
                        />
                    </Show>

                    <Show when={!projectStats.loading && getRoute() === ROUTE_DETAILS}>
                        <ProjectStatDetails
                            data={projectStats()}
                            navigate={() => setRoute(ROUTE_LIST)}
                            createChart={() => setChart(createChart(projectStats()))}
                        />
                    </Show>
                </section>
            </main>

            <Footer/>
        </div>
    );
};

export default App;
