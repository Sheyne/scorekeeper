--
-- Name: tysiac_games; Type: TABLE; Schema: public; Owner: iedftdnfgxrqgs
--

CREATE TABLE public.tysiac_games (
    id integer NOT NULL,
    player_1 character varying NOT NULL,
    player_2 character varying NOT NULL,
    player_3 character varying NOT NULL
);

--
-- Name: tysiac_games_id_seq; Type: SEQUENCE; Schema: public; Owner: iedftdnfgxrqgs
--

CREATE SEQUENCE public.tysiac_games_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tysiac_games_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: iedftdnfgxrqgs
--

ALTER SEQUENCE public.tysiac_games_id_seq OWNED BY public.tysiac_games.id;


--
-- Name: tysiac_scores; Type: TABLE; Schema: public; Owner: iedftdnfgxrqgs
--

CREATE TABLE public.tysiac_scores (
    game_id integer,
    index integer NOT NULL,
    player_1 integer,
    player_2 integer,
    player_3 integer
);


--
-- Name: tysiac_scores_index_seq; Type: SEQUENCE; Schema: public; Owner: iedftdnfgxrqgs
--

CREATE SEQUENCE public.tysiac_scores_index_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tysiac_scores_index_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: iedftdnfgxrqgs
--

ALTER SEQUENCE public.tysiac_scores_index_seq OWNED BY public.tysiac_scores.index;


--
-- Name: tysiac_games id; Type: DEFAULT; Schema: public; Owner: iedftdnfgxrqgs
--

ALTER TABLE ONLY public.tysiac_games ALTER COLUMN id SET DEFAULT nextval('public.tysiac_games_id_seq'::regclass);


--
-- Name: tysiac_scores index; Type: DEFAULT; Schema: public; Owner: iedftdnfgxrqgs
--

ALTER TABLE ONLY public.tysiac_scores ALTER COLUMN index SET DEFAULT nextval('public.tysiac_scores_index_seq'::regclass);


--
-- Data for Name: tysiac_games; Type: TABLE DATA; Schema: public; Owner: iedftdnfgxrqgs
--

COPY public.tysiac_games (id, player_1, player_2, player_3) FROM stdin;
4	Alissa	Daniel	Sheyne
5	Alissa	Daniel	Sheyne
6	Alissa	Sheyne	Daniel
7	Alissa	Sheyne	Daniel
8	Alissa	Daniel	Sheyne
9	Alissa	Daniel	Sheyne
10	Alissa	Daniel	Sheyne
11	Alissa	Daniel	Sheyne
12	Alissa	Daniel	Sheyne
13	Alissa	Daniel	Sheyne
14	Meghan	Daniel	Sheyne
15	Meghan	Sheyne	Alissa
16	Meghan	Sheyne	Nick
17	Nick	Sheyne	Meghan
18	Meghan	Alissa	Sheyne
19	Alissa	Sheyne	Meghan
20	Meghan	Tamara	Sheyne
21	Sheyne	Meghan	Tamara
22	Sheyne	Meghan	Tamara
23	Meghan	Alissa	Sheyne
24	Daniel	Sheyne	Alissa
25	Sheyne	Alissa	Daniel
\.


--
-- Data for Name: tysiac_scores; Type: TABLE DATA; Schema: public; Owner: iedftdnfgxrqgs
--

COPY public.tysiac_scores (game_id, index, player_1, player_2, player_3) FROM stdin;
4	21	105	40	15
4	22	40	-105	80
4	23	50	15	140
4	24	100	120	15
4	25	0	0	120
4	26	0	130	0
4	27	0	0	135
4	28	-105	95	25
4	29	55	-105	65
4	30	25	100	0
4	31	0	65	140
4	32	15	15	145
4	33	0	0	120
5	34	0	175	0
5	35	0	150	0
5	36	50	0	50
5	37	25	55	-160
5	38	55	30	-105
5	39	215	45	25
5	40	50	50	0
5	41	0	-105	40
5	42	0	50	130
5	43	0	-105	30
5	44	0	0	250
5	45	0	125	20
5	46	-105	60	0
5	47	0	65	-105
5	48	-105	40	20
5	49	0	200	0
5	50	0	0	105
5	51	0	165	0
6	52	25	15	120
6	53	-105	30	40
6	54	0	30	130
6	55	0	50	50
6	56	50	0	50
6	57	105	0	0
6	58	0	50	50
6	59	100	-110	35
6	60	25	-105	20
6	61	0	0	110
6	62	0	-105	50
6	63	0	70	-110
6	64	25	-105	50
6	65	25	120	15
6	66	150	0	30
6	67	135	45	90
6	68	40	15	-105
6	69	15	0	130
6	70	0	115	40
6	71	70	-110	0
6	72	45	110	60
6	73	0	0	145
7	74	0	195	15
7	75	0	50	50
7	76	15	110	30
7	77	75	110	0
7	78	0	0	200
7	79	35	25	200
7	80	25	-110	0
7	81	45	110	15
7	82	-175	120	0
7	83	20	0	135
7	84	-110	25	15
7	85	50	110	40
7	86	40	0	110
7	87	0	135	25
7	88	0	0	165
8	89	-105	45	25
8	90	160	25	0
8	91	30	15	120
8	92	-105	70	0
8	93	145	0	25
8	94	0	50	50
8	95	25	130	55
8	96	0	25	-110
8	97	0	140	40
8	98	0	135	20
8	99	200	0	0
8	100	150	15	15
8	101	30	25	-115
8	102	75	-110	0
8	103	0	40	-105
8	104	120	30	55
8	105	50	50	0
8	106	15	105	0
8	107	30	35	110
8	108	35	45	-130
8	109	15	10	110
8	110	10	-110	105
8	111	0	50	0
8	112	0	40	-110
8	113	0	0	120
8	114	0	0	120
8	115	0	20	110
8	116	0	0	120
8	117	0	0	135
8	118	0	-130	30
8	119	0	-130	30
8	120	0	70	-110
8	121	0	65	-105
8	122	0	-120	30
8	123	0	45	100
8	124	0	40	-125
8	125	0	65	-115
8	126	0	15	135
8	127	-125	0	60
8	128	55	80	0
8	129	30	0	120
8	130	40	0	-105
8	131	-120	0	25
8	132	30	0	-105
8	133	90	0	115
8	134	120	0	0
9	135	15	105	70
9	136	85	15	105
9	137	0	110	65
9	138	120	130	0
9	139	120	25	25
9	140	40	115	0
9	141	0	0	110
9	142	0	15	-160
9	143	40	135	15
9	144	50	125	0
9	145	30	40	-140
9	146	60	30	-105
9	147	15	35	85
9	148	0	0	-160
9	149	-120	0	15
9	150	115	0	30
9	151	0	120	0
10	152	0	15	170
10	153	45	0	-120
10	154	130	0	40
10	155	0	180	0
10	156	120	30	65
10	157	30	20	-110
10	158	120	0	-170
10	159	50	50	0
10	160	110	120	15
10	161	55	120	15
10	162	0	120	50
10	163	0	25	175
10	164	120	20	25
10	165	-105	35	40
10	166	205	0	0
10	167	0	20	-115
10	168	0	60	-115
10	169	0	50	-110
10	170	0	135	0
11	171	50	50	0
11	172	35	30	110
11	173	0	180	15
11	174	0	0	120
11	175	40	115	30
11	176	50	0	50
11	177	40	35	140
11	178	0	70	-110
11	179	105	0	0
11	180	40	25	115
11	181	-220	0	10
11	182	25	-110	0
11	183	105	10	15
11	184	0	-115	25
11	185	15	115	95
11	186	115	0	45
11	187	0	50	145
11	188	0	130	70
11	189	-115	20	5
11	190	0	0	120
12	190	50	0	50
12	191	15	175	-135
12	192	0	15	125
12	193	0	110	110
12	194	140	30	0
12	195	85	-105	35
12	196	50	0	50
12	197	200	0	0
12	198	25	15	110
12	199	50	0	130
12	200	110	25	105
12	201	15	120	60
12	202	120	15	20
12	203	20	80	0
12	204	120	0	0
13	205	25	105	25
13	206	40	30	135
13	207	0	170	50
13	208	25	115	0
13	209	-120	40	0
13	210	0	155	0
13	211	110	35	0
13	212	0	0	120
13	213	50	50	0
13	214	115	45	15
13	215	35	110	0
13	216	0	25	135
13	217	15	0	110
13	218	25	0	120
13	219	-115	0	0
13	220	0	0	-110
13	221	25	0	115
13	222	120	0	40
13	223	130	0	25
13	224	15	-110	70
13	225	0	0	150
14	226	65	0	150
14	227	15	120	130
14	228	15	105	30
14	229	15	40	110
14	230	180	0	0
14	231	120	30	100
14	232	130	0	15
14	233	0	-200	15
14	234	20	45	120
14	235	65	130	15
14	236	15	15	110
14	237	0	120	0
14	238	-110	35	85
14	239	0	0	0
14	240	0	120	0
14	241	-120	0	0
14	242	0	0	120
15	243	0	-110	70
15	244	0	-105	65
15	245	15	0	-110
15	246	95	40	110
15	247	25	120	0
15	248	0	120	0
15	249	75	15	115
15	250	180	0	0
15	251	35	15	105
15	252	0	-110	110
15	253	20	125	15
15	254	0	-105	25
15	255	50	-105	70
15	256	0	120	0
15	257	20	120	25
15	258	0	25	230
15	259	0	130	15
15	260	160	85	35
15	261	45	200	0
15	262	0	0	120
16	263	15	140	15
16	264	-105	60	10
16	265	0	125	35
16	266	195	25	25
16	267	15	30	135
16	268	0	260	0
16	269	35	25	-180
16	270	-115	45	45
16	271	120	70	0
16	272	30	-120	45
16	273	-120	40	20
16	274	120	35	50
16	275	15	15	125
16	276	30	0	200
16	277	0	-115	50
16	278	50	15	-115
16	279	105	0	0
16	280	150	25	0
16	281	45	-110	35
16	282	25	0	-120
16	283	140	25	30
16	284	-110	0	50
16	285	15	35	120
16	286	75	35	-110
16	287	0	55	120
16	288	55	-105	110
16	289	30	140	30
16	290	65	15	140
16	291	0	235	0
17	292	0	130	0
17	293	-115	15	45
17	294	-115	15	50
17	295	150	25	15
17	296	105	0	0
17	297	30	25	145
17	298	35	130	15
17	299	-105	0	55
18	300	-105	60	45
18	301	25	35	160
18	302	30	30	105
18	303	-110	25	40
18	304	115	25	45
18	305	15	30	-105
18	306	30	115	45
18	307	240	0	25
18	308	30	15	125
18	309	65	30	120
18	310	-125	15	75
18	311	20	115	40
18	312	60	0	145
18	313	-120	0	15
18	314	0	0	120
19	315	115	50	0
19	316	145	35	-175
19	317	80	0	-105
19	318	35	150	145
19	319	90	30	-100
19	320	30	-115	25
19	321	130	30	15
19	322	50	-120	0
19	323	30	-120	25
19	324	35	125	30
20	325	0	0	160
20	326	40	25	-110
20	327	20	200	0
20	328	-105	70	0
20	329	40	110	40
20	330	80	0	-110
20	331	25	15	130
20	332	0	110	15
20	333	75	0	-115
20	334	15	-110	45
20	335	25	160	15
20	336	0	160	15
20	337	140	40	0
20	338	60	15	-110
20	339	65	0	110
20	340	95	65	140
20	341	145	20	-120
20	342	0	0	120
20	343	60	0	-115
20	344	-105	0	55
20	345	50	0	105
20	346	55	0	145
20	347	0	0	120
20	348	0	120	0
21	349	25	40	-105
21	350	140	30	25
21	351	0	180	15
21	352	-110	80	15
21	353	-105	30	160
21	354	0	220	0
21	355	0	120	45
21	356	105	0	0
21	357	15	115	40
21	358	110	0	25
21	359	20	40	-105
21	360	30	25	-120
21	361	120	0	0
21	362	-105	0	0
21	363	0	0	120
21	364	155	0	0
21	365	0	0	240
21	366	45	0	115
21	367	0	120	0
22	368	130	45	15
22	369	-115	50	30
22	370	15	115	30
22	371	50	25	105
22	372	175	0	0
22	373	25	35	-110
22	374	155	0	0
22	375	125	0	85
23	376	0	45	250
23	377	40	-110	130
23	378	-110	0	25
23	379	15	40	-115
23	380	0	140	50
23	381	0	0	115
23	382	0	0	175
23	383	25	0	150
23	384	15	125	35
23	385	25	0	185
24	386	0	120	0
24	387	140	0	0
24	388	140	35	30
24	389	0	-110	50
24	390	25	0	230
24	391	140	45	30
24	392	0	20	140
24	393	45	-120	0
24	394	200	15	25
24	395	140	0	0
24	396	175	145	15
25	397	-120	40	25
25	398	50	25	200
25	399	0	-115	60
25	400	25	115	50
25	401	-105	45	0
25	402	35	180	30
25	403	150	45	-115
25	404	-115	60	0
25	405	120	0	0
25	406	0	50	-105
25	407	0	25	135
25	408	30	-115	160
25	409	25	180	40
25	410	120	0	35
25	411	120	0	0
25	412	-110	0	60
25	413	190	35	0
25	414	0	180	0
25	415	-130	0	0
25	416	-120	45	0
25	417	-110	0	20
25	418	15	85	15
25	419	160	0	0
25	420	15	120	0
\.


--
-- Name: tysiac_games_id_seq; Type: SEQUENCE SET; Schema: public; Owner: iedftdnfgxrqgs
--

SELECT pg_catalog.setval('public.tysiac_games_id_seq', 25, true);


--
-- Name: tysiac_scores_index_seq; Type: SEQUENCE SET; Schema: public; Owner: iedftdnfgxrqgs
--

SELECT pg_catalog.setval('public.tysiac_scores_index_seq', 420, true);


--
-- Name: tysiac_games tysiac_games_pkey; Type: CONSTRAINT; Schema: public; Owner: iedftdnfgxrqgs
--

ALTER TABLE ONLY public.tysiac_games
    ADD CONSTRAINT tysiac_games_pkey PRIMARY KEY (id);


--
-- Name: tysiac_scores tysiac_scores_game_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: iedftdnfgxrqgs
--

ALTER TABLE ONLY public.tysiac_scores
    ADD CONSTRAINT tysiac_scores_game_id_fkey FOREIGN KEY (game_id) REFERENCES public.tysiac_games(id);

